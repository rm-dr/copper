use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use ufo_pipeline::{
	api::{InitNodeError, Node, NodeInfo, NodeState, PipelineData, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

use crate::data::{UFOData, UFODataStub};

pub struct Constant {
	inputs: BTreeMap<PipelinePortID, UFODataStub>,
	outputs: BTreeMap<PipelinePortID, UFODataStub>,
	value: UFOData,
}

impl Constant {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 1 });
		}

		let value = if let Some(value) = params.get("value") {
			match value {
				NodeParameterValue::Data(data) => data.clone(),
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "value".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "value".into(),
			});
		};

		Ok(Self {
			inputs: BTreeMap::new(),
			outputs: BTreeMap::from([(PipelinePortID::new("out"), value.as_stub())]),
			value,
		})
	}
}

impl NodeInfo<UFOData> for Constant {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <UFOData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <UFOData as PipelineData>::DataStubType> {
		&self.outputs
	}
}

impl Node<UFOData> for Constant {
	fn get_info(&self) -> &dyn NodeInfo<UFOData> {
		self
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		_target_port: PipelinePortID,
		_input_data: UFOData,
	) -> Result<(), RunNodeError> {
		unreachable!("Constant nodes do not take input.")
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, UFOData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		send_data(PipelinePortID::new("out"), self.value.clone())?;
		Ok(NodeState::Done)
	}
}
