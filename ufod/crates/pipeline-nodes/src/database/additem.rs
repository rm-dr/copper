use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use std::{
	fmt::Debug,
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
	api::{PipelineNode, PipelineNodeError, PipelineNodeState},
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	helpers::DataSource,
	nodetype::{UFONodeType, UFONodeTypeError},
	traits::UFONode,
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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct AddItemConfig {
	#[serde(default)]
	allow_non_unique: bool,
}

pub struct AddItem {
	dataset: Arc<LocalDataset>,

	class: ClassHandle,
	attrs: Vec<AttrInfo>,
	config: AddItemConfig,

	data: Vec<Option<DataHold>>,
}

impl AddItem {
	pub fn new(
		ctx: &<Self as PipelineNode>::NodeContext,
		class: ClassHandle,
		attrs: Vec<AttrInfo>,
		config: AddItemConfig,
	) -> Self {
		let data = attrs.iter().map(|_| None).collect();
		AddItem {
			dataset: ctx.dataset.clone(),

			class,
			attrs,
			data,
			config,
		}
	}
}

impl PipelineNode for AddItem {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineNodeError> {
		assert!(port < self.attrs.len());
		match data {
			UFOData::Bytes { mime, source } => {
				let x = &self.attrs[port];
				match x.data_type {
					MetastoreDataStub::Binary => {
						if self.data[port].is_none() {
							self.data[port] = Some(DataHold::Binary(DataSource::Uninitialized));
						}

						match &mut self.data[port] {
							Some(DataHold::Binary(reader)) => {
								reader.consume(mime, source);
							}
							_ => unreachable!(),
						}
					}

					MetastoreDataStub::Blob => {
						if self.data[port].is_none() {
							self.data[port] = Some(DataHold::BlobWriting {
								reader: DataSource::Uninitialized,
								writer: Some(
									block_on(self.dataset.new_blob(&mime))
										.map_err(|e| PipelineNodeError::Other(Box::new(e)))?,
								),
							});
						}

						match &mut self.data[port] {
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
				self.data[port] = Some(DataHold::Static(x));
			}
		};
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineNodeError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineNodeError>,
	{
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
					if self.config.allow_non_unique {
						send_data(
							0,
							UFOData::None(UFODataStub::Reference { class: self.class }),
						)?;
					} else {
						return Err(PipelineNodeError::Other(Box::new(err)));
					}
				}
				_ => return Err(PipelineNodeError::Other(Box::new(err))),
			},
		}

		Ok(PipelineNodeState::Done)
	}
}

impl UFONode for AddItem {
	fn n_inputs(stub: &UFONodeType, ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { class, .. } => {
				let class = if let Some(c) = block_on(ctx.dataset.get_class_by_name(&class[..]))? {
					c
				} else {
					return Err(UFONodeTypeError::NoSuchClass(class.clone()));
				};

				let attrs = block_on(ctx.dataset.class_get_attrs(class.handle))?;
				attrs.len()
			}
			_ => unreachable!(),
		})
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> Result<bool, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { .. } => {
				Self::input_default_type(stub, ctx, input_idx)? == input_type
			}
			_ => unreachable!(),
		})
	}

	fn input_with_name(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { class, .. } => {
				let class = if let Some(c) = block_on(ctx.dataset.get_class_by_name(&class[..]))? {
					c
				} else {
					return Err(UFONodeTypeError::NoSuchClass(class.clone()));
				};

				let attrs = block_on(ctx.dataset.class_get_attrs(class.handle))?;
				attrs
					.into_iter()
					.enumerate()
					.find(|(_, a)| PipelinePortID::new(&a.name) == *input_name)
					.map(|(i, _)| i)
			}
			_ => unreachable!(),
		})
	}

	fn input_default_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { class, .. } => {
				let class = if let Some(c) = block_on(ctx.dataset.get_class_by_name(&class[..]))? {
					c
				} else {
					return Err(UFONodeTypeError::NoSuchClass(class.clone()));
				};

				let attrs = block_on(ctx.dataset.class_get_attrs(class.handle))?;
				attrs.into_iter().nth(input_idx).unwrap().data_type.into()
			}
			_ => unreachable!(),
		})
	}

	fn n_outputs(stub: &UFONodeType, ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { class, .. } => {
				let class = if let Some(c) = block_on(ctx.dataset.get_class_by_name(&class[..]))? {
					c
				} else {
					return Err(UFONodeTypeError::NoSuchClass(class.clone()));
				};

				let attrs = block_on(ctx.dataset.class_get_attrs(class.handle))?;
				attrs.len()
			}
			_ => unreachable!(),
		})
	}

	fn output_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		output_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { class, .. } => {
				assert!(output_idx == 0);

				let class = if let Some(c) = block_on(ctx.dataset.get_class_by_name(&class[..]))? {
					c
				} else {
					return Err(UFONodeTypeError::NoSuchClass(class.clone()));
				};

				UFODataStub::Reference {
					class: class.handle,
				}
			}
			_ => unreachable!(),
		})
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::AddItem { .. } => match output_name.id().as_str() {
				"added_item" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		})
	}
}
