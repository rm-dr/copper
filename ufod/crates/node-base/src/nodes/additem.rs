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
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineNode, PipelineNodeError,
		PipelineNodeState,
	},
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

pub struct AddItem {
	inputs: Vec<NodeInputInfo<<UFOData as PipelineData>::DataStubType>>,

	dataset: Arc<LocalDataset>,
	class: ClassHandle,
	attrs: Vec<AttrInfo>,
	error_non_unique: bool,

	data: Vec<Option<DataHold>>,
}

impl AddItem {
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Self {
		if params.len() != 3 {
			panic!()
		}

		let error_non_unique: bool = if let Some(value) = params.get("error_non_unique") {
			match value {
				NodeParameterValue::Boolean(x) => *x,
				_ => panic!(),
			}
		} else {
			panic!()
		};

		let class: ClassHandle = if let Some(value) = params.get("class") {
			match value {
				NodeParameterValue::String(s) => {
					let x = block_on(ctx.dataset.get_class_by_name(s));
					match x {
						Ok(Some(x)) => x.handle,
						Ok(None) => {
							panic!()
						}
						Err(_) => {
							panic!()
						}
					}
				}
				_ => panic!(),
			}
		} else {
			panic!()
		};

		let mut attrs: Vec<AttrInfo> = Vec::new();
		if let Some(taglist) = params.get("tags") {
			match taglist {
				NodeParameterValue::List(list) => {
					for t in list {
						match t {
							NodeParameterValue::String(s) => {
								let x = block_on(ctx.dataset.get_attr_by_name(class, s));
								match x {
									Ok(Some(x)) => attrs.push(x),
									Ok(None) => {
										panic!()
									}
									Err(_) => {
										panic!()
									}
								}
							}
							_ => panic!(),
						}
					}
				}
				_ => panic!(),
			}
		} else {
			panic!()
		}

		let data = attrs.iter().map(|_| None).collect();
		AddItem {
			inputs: attrs
				.iter()
				.map(|x| NodeInputInfo {
					name: PipelinePortID::new(&x.name),
					accepts_type: x.data_type.into(),
				})
				.collect(),

			dataset: ctx.dataset.clone(),

			class,
			attrs,
			data,
			error_non_unique,
		}
	}
}

impl PipelineNode<UFOData> for AddItem {
	fn inputs(&self) -> &[NodeInputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.inputs
	}

	fn outputs(&self) -> &[NodeOutputInfo<<UFOData as PipelineData>::DataStubType>] {
		&[]
	}

	fn take_input(
		&mut self,
		target_port: usize,
		input_data: UFOData,
	) -> Result<(), PipelineNodeError> {
		assert!(target_port < self.attrs.len());
		match input_data {
			UFOData::Bytes { mime, source } => {
				let x = &self.attrs[target_port];
				match x.data_type {
					MetastoreDataStub::Binary => {
						if self.data[target_port].is_none() {
							self.data[target_port] =
								Some(DataHold::Binary(DataSource::Uninitialized));
						}

						match &mut self.data[target_port] {
							Some(DataHold::Binary(reader)) => {
								reader.consume(mime, source);
							}
							_ => unreachable!(),
						}
					}

					MetastoreDataStub::Blob => {
						if self.data[target_port].is_none() {
							self.data[target_port] = Some(DataHold::BlobWriting {
								reader: DataSource::Uninitialized,
								writer: Some(
									block_on(self.dataset.new_blob(&mime))
										.map_err(|e| PipelineNodeError::Other(Box::new(e)))?,
								),
							});
						}

						match &mut self.data[target_port] {
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
				self.data[target_port] = Some(DataHold::Static(x));
			}
		};
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
		for i in &mut self.data {
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
								.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
							std::mem::swap(i, &mut Some(DataHold::BlobDone(x)));
						}
					}

					DataSource::File { file, .. } => {
						std::io::copy(file, writer.as_mut().unwrap())?;
						let x = block_on(self.dataset.finish_blob(writer.take().unwrap()))
							.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
						std::mem::swap(i, &mut Some(DataHold::BlobDone(x)));
					}
				},
				Some(_) => {}
				None => return Ok(PipelineNodeState::Pending("waiting for data")),
			}
		}

		let mut attrs = Vec::new();
		for (attr, data) in self.attrs.iter().zip(self.data.iter_mut()) {
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
		let res = block_on(self.dataset.add_item(self.class, attrs));

		match res {
			Ok(item) => {
				send_data(
					0,
					UFOData::Reference {
						class: self.class,
						item,
					},
				)?;
			}
			Err(err) => match err {
				MetastoreError::UniqueViolated => {
					if self.error_non_unique {
						return Err(PipelineNodeError::Other(Box::new(err)));
					} else {
						send_data(
							0,
							UFOData::None {
								data_type: UFODataStub::Reference { class: self.class },
							},
						)?;
					}
				}
				_ => return Err(PipelineNodeError::Other(Box::new(err))),
			},
		}

		Ok(PipelineNodeState::Done)
	}
}
