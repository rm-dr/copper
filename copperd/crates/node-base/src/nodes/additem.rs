use copper_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore, BlobstoreTmpWriter},
		meta::{AttrInfo, Metastore},
	},
	data::{MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::ClassHandle,
};
use copper_ds_impl::local::LocalDataset;
use copper_pipeline::{
	api::{InitNodeError, Node, NodeInfo, NodeState, PipelineData, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{Read, Write},
	sync::Arc,
};

use crate::{
	data::{CopperData, CopperDataStub},
	helpers::DataSource,
	CopperContext,
};

enum DataHold {
	Static(CopperData),
	Binary(DataSource),
	BlobWriting {
		reader: DataSource,
		writer: Option<BlobstoreTmpWriter>,
	},
	BlobDone(BlobHandle),
}

pub struct AddItemInfo {
	outputs: BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType>,
	inputs: BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType>,

	class: ClassHandle,
	attrs: BTreeMap<PipelinePortID, AttrInfo>,
	error_non_unique: bool,

	data: BTreeMap<PipelinePortID, Option<DataHold>>,
}

impl AddItemInfo {
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 2 {
			return Err(InitNodeError::BadParameterCount { expected: 2 });
		}

		let error_non_unique: bool = if let Some(value) = params.get("error_non_unique") {
			match value {
				NodeParameterValue::Boolean(x) => *x,
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "error_non_unique".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "error_non_unique".into(),
			});
		};

		let class: ClassHandle = if let Some(value) = params.get("class") {
			match value {
				NodeParameterValue::String(s) => {
					let x = block_on(ctx.dataset.get_class_by_name(s));
					match x {
						Ok(Some(x)) => x.handle,
						Ok(None) => {
							return Err(InitNodeError::BadParameterOther {
								param_name: "class".into(),
								message: "No such class".into(),
							})
						}
						Err(e) => return Err(InitNodeError::Other(Box::new(e))),
					}
				}
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "class".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "class".into(),
			});
		};

		let attrs: BTreeMap<PipelinePortID, AttrInfo> =
			block_on(ctx.dataset.class_get_attrs(class))
				.unwrap()
				.into_iter()
				.map(|x| (PipelinePortID::new(&x.name), x))
				.collect();

		let data = attrs.keys().map(|k| (k.clone(), None)).collect();

		Ok(Self {
			outputs: BTreeMap::from([(
				PipelinePortID::new("new_item"),
				CopperDataStub::Reference { class },
			)]),

			inputs: attrs
				.iter()
				.map(|(k, v)| (k.clone(), v.data_type.into()))
				.collect(),

			class,
			attrs,
			data,
			error_non_unique,
		})
	}
}

impl NodeInfo<CopperData> for AddItemInfo {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, CopperDataStub> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, CopperDataStub> {
		&self.outputs
	}
}

pub struct AddItem {
	dataset: Arc<LocalDataset>,
	info: AddItemInfo,
}

impl AddItem {
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		Ok(AddItem {
			dataset: ctx.dataset.clone(),
			info: AddItemInfo::new(ctx, params)?,
		})
	}
}

impl Node<CopperData> for AddItem {
	fn get_info(&self) -> &dyn NodeInfo<CopperData> {
		&self.info
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: CopperData,
	) -> Result<(), RunNodeError> {
		assert!(
			self.info.inputs.contains_key(&target_port),
			"Received data on invalid port {target_port}"
		);

		match input_data {
			CopperData::Bytes { mime, source } => {
				let x = &self.info.attrs.get(&target_port).unwrap();
				match x.data_type {
					MetastoreDataStub::Binary => {
						if self.info.data.get(&target_port).unwrap().is_none() {
							*self.info.data.get_mut(&target_port).unwrap() =
								Some(DataHold::Binary(DataSource::Uninitialized));
						}

						match self.info.data.get_mut(&target_port).unwrap() {
							Some(DataHold::Binary(reader)) => {
								reader.consume(mime, source);
							}
							_ => unreachable!(),
						}
					}

					MetastoreDataStub::Blob => {
						if self.info.data.get(&target_port).unwrap().is_none() {
							*self.info.data.get_mut(&target_port).unwrap() =
								Some(DataHold::BlobWriting {
									reader: DataSource::Uninitialized,
									writer: Some(
										block_on(self.dataset.new_blob(&mime))
											.map_err(|e| RunNodeError::Other(Box::new(e)))?,
									),
								});
						}

						match self.info.data.get_mut(&target_port).unwrap() {
							Some(DataHold::BlobWriting { reader, .. }) => {
								reader.consume(mime, source);
							}
							_ => unreachable!(),
						}
					}
					_ => unreachable!(),
				}
			}

			x => {
				*self.info.data.get_mut(&target_port).unwrap() = Some(DataHold::Static(x));
			}
		};
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		for hold in self.info.data.values_mut() {
			match hold {
				Some(DataHold::BlobWriting { reader, writer }) => match reader {
					DataSource::Uninitialized => {
						unreachable!()
					}

					DataSource::Binary { data, is_done, .. } => {
						while let Some(data) = data.pop_front() {
							writer.as_mut().unwrap().write_all(&data[..])?;
						}

						if *is_done {
							let x = block_on(self.dataset.finish_blob(writer.take().unwrap()))
								.map_err(|e| RunNodeError::Other(Box::new(e)))?;
							std::mem::swap(hold, &mut Some(DataHold::BlobDone(x)));
						}
					}

					DataSource::File { file, .. } => {
						std::io::copy(file, writer.as_mut().unwrap())?;
						let x = block_on(self.dataset.finish_blob(writer.take().unwrap()))
							.map_err(|e| RunNodeError::Other(Box::new(e)))?;
						std::mem::swap(hold, &mut Some(DataHold::BlobDone(x)));
					}
				},
				Some(_) => {}
				None => return Ok(NodeState::Pending("waiting for data")),
			}
		}

		let mut attrs = Vec::new();
		for (port, attr) in self.info.attrs.iter() {
			let data = match self.info.data.get_mut(port).unwrap() {
				Some(DataHold::Binary(x)) => match x {
					DataSource::Binary {
						mime,
						data,
						is_done: true,
					} => MetastoreData::Binary {
						mime: mime.clone(),
						data: {
							let mut v = Vec::new();
							for d in data {
								v.extend(&**d);
							}
							Arc::new(v)
						},
					},

					DataSource::File { mime, file } => MetastoreData::Binary {
						mime: mime.clone(),
						data: {
							let mut v = Vec::new();
							file.read_to_end(&mut v)?;
							Arc::new(v)
						},
					},

					_ => unreachable!(),
				},

				Some(DataHold::Static(x)) => x.as_db_data().unwrap(),
				Some(DataHold::BlobDone(handle)) => MetastoreData::Blob { handle: *handle },
				_ => unreachable!(),
			};
			attrs.push((attr.handle, data));
		}
		let res = block_on(self.dataset.add_item(self.info.class, attrs));

		match res {
			Ok(item) => {
				send_data(
					PipelinePortID::new("new_item"),
					CopperData::Reference {
						class: self.info.class,
						item,
					},
				)?;
			}
			Err(err) => match err {
				MetastoreError::UniqueViolated => {
					if self.info.error_non_unique {
						return Err(RunNodeError::Other(Box::new(err)));
					} else {
						send_data(
							PipelinePortID::new("new_item"),
							CopperData::None {
								data_type: CopperDataStub::Reference {
									class: self.info.class,
								},
							},
						)?;
					}
				}
				_ => return Err(RunNodeError::Other(Box::new(err))),
			},
		}

		Ok(NodeState::Done)
	}
}
