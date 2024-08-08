use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{Read, Write},
	sync::Arc,
};
use ufo_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore, BlobstoreTmpWriter},
		meta::{AttrInfo, Metastore},
	},
	data::{MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::ClassHandle,
};
use ufo_ds_impl::local::LocalDataset;
use ufo_pipeline::{
	api::{InitNodeError, NodeInfo, PipelineData, PipelineNode, PipelineNodeState, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	helpers::DataSource,
	UFOContext,
};

enum DataHold {
	Static(UFOData),
	Binary(DataSource),
	BlobWriting {
		reader: DataSource,
		writer: Option<BlobstoreTmpWriter>,
	},
	BlobDone(BlobHandle),
}

pub struct AddItemInfo {
	inputs: Vec<(PipelinePortID, <UFOData as PipelineData>::DataStubType)>,

	class: ClassHandle,
	attrs: Vec<AttrInfo>,
	error_non_unique: bool,

	data: Vec<Option<DataHold>>,
}

impl AddItemInfo {
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
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

		let attrs: Vec<AttrInfo> = block_on(ctx.dataset.class_get_attrs(class)).unwrap();
		let data = attrs.iter().map(|_| None).collect();

		Ok(Self {
			inputs: attrs
				.iter()
				.map(|x| (PipelinePortID::new(&x.name), x.data_type.into()))
				.collect(),

			class,
			attrs,
			data,
			error_non_unique,
		})
	}
}

impl NodeInfo<UFOData> for AddItemInfo {
	fn inputs(&self) -> &[(PipelinePortID, UFODataStub)] {
		&self.inputs
	}

	fn outputs(&self) -> &[(PipelinePortID, UFODataStub)] {
		&[]
	}
}

pub struct AddItem {
	dataset: Arc<LocalDataset>,
	info: AddItemInfo,
}

impl AddItem {
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, InitNodeError> {
		Ok(AddItem {
			dataset: ctx.dataset.clone(),
			info: AddItemInfo::new(ctx, params)?,
		})
	}
}

impl PipelineNode<UFOData> for AddItem {
	fn get_info(&self) -> &dyn NodeInfo<UFOData> {
		&self.info
	}

	fn take_input(&mut self, target_port: usize, input_data: UFOData) -> Result<(), RunNodeError> {
		assert!(target_port < self.info.attrs.len());
		match input_data {
			UFOData::Bytes { mime, source } => {
				let x = &self.info.attrs[target_port];
				match x.data_type {
					MetastoreDataStub::Binary => {
						if self.info.data[target_port].is_none() {
							self.info.data[target_port] =
								Some(DataHold::Binary(DataSource::Uninitialized));
						}

						match &mut self.info.data[target_port] {
							Some(DataHold::Binary(reader)) => {
								reader.consume(mime, source);
							}
							_ => unreachable!(),
						}
					}

					MetastoreDataStub::Blob => {
						if self.info.data[target_port].is_none() {
							self.info.data[target_port] = Some(DataHold::BlobWriting {
								reader: DataSource::Uninitialized,
								writer: Some(
									block_on(self.dataset.new_blob(&mime))
										.map_err(|e| RunNodeError::Other(Box::new(e)))?,
								),
							});
						}

						match &mut self.info.data[target_port] {
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
				self.info.data[target_port] = Some(DataHold::Static(x));
			}
		};
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), RunNodeError>,
	) -> Result<PipelineNodeState, RunNodeError> {
		for i in &mut self.info.data {
			match i {
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
							std::mem::swap(i, &mut Some(DataHold::BlobDone(x)));
						}
					}

					DataSource::File { file, .. } => {
						std::io::copy(file, writer.as_mut().unwrap())?;
						let x = block_on(self.dataset.finish_blob(writer.take().unwrap()))
							.map_err(|e| RunNodeError::Other(Box::new(e)))?;
						std::mem::swap(i, &mut Some(DataHold::BlobDone(x)));
					}
				},
				Some(_) => {}
				None => return Ok(PipelineNodeState::Pending("waiting for data")),
			}
		}

		let mut attrs = Vec::new();
		for (attr, data) in self.info.attrs.iter().zip(self.info.data.iter_mut()) {
			let data = match data.as_mut().unwrap() {
				DataHold::Binary(x) => match x {
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

				DataHold::Static(x) => x.as_db_data().unwrap(),
				DataHold::BlobDone(handle) => MetastoreData::Blob { handle: *handle },
				_ => unreachable!(),
			};
			attrs.push((attr.handle, data));
		}
		let res = block_on(self.dataset.add_item(self.info.class, attrs));

		match res {
			Ok(item) => {
				send_data(
					0,
					UFOData::Reference {
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
							0,
							UFOData::None {
								data_type: UFODataStub::Reference {
									class: self.info.class,
								},
							},
						)?;
					}
				}
				_ => return Err(RunNodeError::Other(Box::new(err))),
			},
		}

		Ok(PipelineNodeState::Done)
	}
}
