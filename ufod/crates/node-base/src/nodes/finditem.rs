use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use ufo_ds_core::{
	api::meta::{AttrInfo, Metastore},
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
	UFOContext,
};

pub struct FindItem {
	inputs: Vec<NodeInputInfo<<UFOData as PipelineData>::DataStubType>>,
	outputs: Vec<NodeOutputInfo<<UFOData as PipelineData>::DataStubType>>,

	dataset: Arc<LocalDataset>,
	class: ClassHandle,
	by_attr: AttrInfo,

	attr_value: Option<UFOData>,
}

impl FindItem {
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, PipelineNodeError> {
		if params.len() != 2 {
			return Err(PipelineNodeError::BadParameterCount { expected: 2 });
		}

		let class: ClassHandle = if let Some(value) = params.get("class") {
			match value {
				NodeParameterValue::String(s) => {
					let x = block_on(ctx.dataset.get_class_by_name(s));
					match x {
						Ok(Some(x)) => x.handle,
						Ok(None) => {
							return Err(PipelineNodeError::BadParameterOther {
								param_name: "class".into(),
								message: "No such class".into(),
							})
						}
						Err(e) => return Err(PipelineNodeError::Other(Box::new(e))),
					}
				}
				_ => {
					return Err(PipelineNodeError::BadParameterType {
						param_name: "class".into(),
					})
				}
			}
		} else {
			return Err(PipelineNodeError::MissingParameter {
				param_name: "class".into(),
			});
		};

		let by_attr: AttrInfo = if let Some(value) = params.get("by_attr") {
			match value {
				NodeParameterValue::String(s) => {
					let x = block_on(ctx.dataset.get_attr_by_name(class, s));
					match x {
						Ok(Some(x)) => x.clone(),
						Ok(None) => {
							return Err(PipelineNodeError::BadParameterOther {
								param_name: "by_attr".into(),
								message: "No such attribute".into(),
							})
						}
						Err(e) => return Err(PipelineNodeError::Other(Box::new(e))),
					}
				}
				_ => {
					return Err(PipelineNodeError::BadParameterType {
						param_name: "by_attr".into(),
					})
				}
			}
		} else {
			return Err(PipelineNodeError::MissingParameter {
				param_name: "by_attr".into(),
			});
		};

		Ok(FindItem {
			inputs: vec![NodeInputInfo {
				name: PipelinePortID::new("attr_value"),
				accepts_type: by_attr.data_type.into(),
			}],

			outputs: vec![NodeOutputInfo {
				name: PipelinePortID::new("found_item"),
				produces_type: UFODataStub::Reference { class },
			}],

			dataset: ctx.dataset.clone(),

			class,
			by_attr,
			attr_value: None,
		})
	}
}

impl PipelineNode<UFOData> for FindItem {
	fn inputs(&self) -> &[NodeInputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.inputs
	}

	fn outputs(&self) -> &[NodeOutputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.outputs
	}

	fn take_input(
		&mut self,
		target_port: usize,
		input_data: UFOData,
	) -> Result<(), PipelineNodeError> {
		assert!(target_port == 0);
		assert!(input_data.as_stub() == self.by_attr.data_type.into());
		self.attr_value = Some(input_data);
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
		if self.attr_value.is_none() {
			return Ok(PipelineNodeState::Pending("waiting for input"));
		}

		let found = block_on(self.dataset.find_item_with_attr(
			self.by_attr.handle,
			self.attr_value.as_ref().unwrap().as_db_data().unwrap(),
		))
		.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;

		if let Some(item) = found {
			send_data(
				0,
				UFOData::Reference {
					class: self.class,
					item,
				},
			)?;
		} else {
			send_data(
				0,
				UFOData::None {
					data_type: UFODataStub::Reference { class: self.class },
				},
			)?;
		}

		return Ok(PipelineNodeState::Done);
	}
}
