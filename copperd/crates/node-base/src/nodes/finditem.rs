use copper_ds_core::{
	api::meta::{AttrInfo, Metastore},
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
use std::{collections::BTreeMap, sync::Arc};

use crate::{
	data::{CopperData, CopperDataStub},
	CopperContext,
};

pub struct FindItemInfo {
	inputs: BTreeMap<PipelinePortID, CopperDataStub>,
	outputs: BTreeMap<PipelinePortID, CopperDataStub>,

	class: ClassHandle,
	by_attr: AttrInfo,
}

impl FindItemInfo {
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 2 {
			return Err(InitNodeError::BadParameterCount { expected: 2 });
		}

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

		let by_attr: AttrInfo = if let Some(value) = params.get("by_attr") {
			match value {
				NodeParameterValue::String(s) => {
					let x = block_on(ctx.dataset.get_attr_by_name(class, s));
					match x {
						Ok(Some(x)) => x.clone(),
						Ok(None) => {
							return Err(InitNodeError::BadParameterOther {
								param_name: "by_attr".into(),
								message: "No such attribute".into(),
							})
						}
						Err(e) => return Err(InitNodeError::Other(Box::new(e))),
					}
				}
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "by_attr".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "by_attr".into(),
			});
		};

		Ok(Self {
			inputs: BTreeMap::from([(PipelinePortID::new("attr_value"), by_attr.data_type.into())]),
			outputs: BTreeMap::from([(
				PipelinePortID::new("found_item"),
				CopperDataStub::Reference { class },
			)]),

			class,
			by_attr,
		})
	}
}

impl NodeInfo<CopperData> for FindItemInfo {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.outputs
	}
}

pub struct FindItem {
	info: FindItemInfo,
	dataset: Arc<LocalDataset>,
	attr_value: Option<CopperData>,
}

impl FindItem {
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 2 {
			return Err(InitNodeError::BadParameterCount { expected: 2 });
		}

		let info = FindItemInfo::new(ctx, params)?;

		Ok(Self {
			info,
			dataset: ctx.dataset.clone(),
			attr_value: None,
		})
	}
}

impl Node<CopperData> for FindItem {
	fn get_info(&self) -> &dyn NodeInfo<CopperData> {
		&self.info
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: CopperData,
	) -> Result<(), RunNodeError> {
		assert!(target_port == PipelinePortID::new("attr_value"));
		assert!(input_data.as_stub() == self.info.by_attr.data_type.into());
		self.attr_value = Some(input_data);
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		if self.attr_value.is_none() {
			return Ok(NodeState::Pending("waiting for input"));
		}

		let found = block_on(self.dataset.find_item_with_attr(
			self.info.by_attr.handle,
			self.attr_value.as_ref().unwrap().as_db_data().unwrap(),
		))
		.map_err(|e| RunNodeError::Other(Box::new(e)))?;

		// TODO: how to handle multiple found items?
		if let Some(item) = found.first() {
			send_data(
				PipelinePortID::new("found_item"),
				CopperData::Reference {
					class: self.info.class,
					item: *item,
				},
			)?;
		} else {
			send_data(
				PipelinePortID::new("found_item"),
				CopperData::None {
					data_type: CopperDataStub::Reference {
						class: self.info.class,
					},
				},
			)?;
		}

		return Ok(NodeState::Done);
	}
}
