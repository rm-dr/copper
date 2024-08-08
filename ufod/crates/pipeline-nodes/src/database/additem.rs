use core::panic;
use std::{collections::VecDeque, fmt::Debug, io::Write, sync::Arc};

use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use ufo_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore, BlobstoreTmpWriter},
		meta::{AttrInfo, Metastore},
	},
	data::MetastoreData,
	errors::MetastoreError,
	handles::ClassHandle,
};
use ufo_ds_impl::local::LocalDataset;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	nodetype::{UFONodeType, UFONodeTypeError},
	traits::UFONode,
	UFOContext,
};

enum DataHold {
	Static(UFOData),
	BlobWriting {
		buffer: VecDeque<Arc<Vec<u8>>>,
		writer: Option<BlobstoreTmpWriter>,
		is_done: bool,
	},
	BlobDone(BlobHandle),
}

impl Debug for DataHold {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			DataHold::Static(data) => write!(f, "Static: {:?}", data),
			DataHold::BlobWriting { .. } => write!(f, "Writing blob"),
			DataHold::BlobDone(h) => write!(f, "Blob done: {h:?}"),
		}
	}
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
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		assert!(port < self.attrs.len());
		match data {
			UFOData::Blob {
				mime: format,
				fragment,
				is_last,
			} => match &mut self.data[port] {
				None => {
					self.data[port] = Some(DataHold::BlobWriting {
						buffer: VecDeque::from([fragment]),
						writer: Some(block_on(self.dataset.new_blob(&format))?),
						is_done: is_last,
					})
				}
				Some(DataHold::BlobWriting {
					buffer, is_done, ..
				}) => {
					buffer.push_back(fragment);
					*is_done = is_last;
				}
				x => panic!("bad input {x:?}"),
			},
			x => {
				self.data[port] = Some(DataHold::Static(x));
			}
		};
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let mut exit = false;
		for i in &mut self.data {
			match i {
				Some(DataHold::BlobWriting {
					buffer,
					writer,
					is_done,
				}) => {
					while let Some(data) = buffer.pop_front() {
						writer.as_mut().unwrap().write(&data[..])?;
					}
					if *is_done {
						let x = block_on(self.dataset.finish_blob(writer.take().unwrap()))?;
						std::mem::swap(i, &mut Some(DataHold::BlobDone(x)));
					}
				}
				Some(_) => {}
				None => exit = true,
			}
		}

		if exit {
			return Ok(PipelineNodeState::Pending("waiting for data"));
		}

		let mut attrs = Vec::new();
		for (attr, data) in self.attrs.iter().zip(self.data.iter_mut()) {
			let data = match data.as_ref().unwrap() {
				DataHold::Static(x) => x.as_db_data().unwrap(),
				DataHold::BlobDone(handle) => MetastoreData::Blob {
					handle: handle.clone(),
				},
				_ => unreachable!(),
			};
			attrs.push((attr.handle, data.into()));
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
						return Err(err.into());
					}
				}
				_ => return Err(err.into()),
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
				attrs.into_iter().count()
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
				attrs.into_iter().count()
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
