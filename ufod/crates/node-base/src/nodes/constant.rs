use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use ufo_pipeline::{
	api::{InitNodeError, NodeInfo, PipelineData, PipelineNode, PipelineNodeState, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

use crate::data::{UFOData, UFODataStub};

pub struct Constant {
	outputs: [(PipelinePortID, UFODataStub); 1],
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
			outputs: [(PipelinePortID::new("out"), value.as_stub())],
			value,
		})
	}
}

impl NodeInfo<UFOData> for Constant {
	fn inputs(&self) -> &[(PipelinePortID, <UFOData as PipelineData>::DataStubType)] {
		&[]
	}

	fn outputs(&self) -> &[(PipelinePortID, <UFOData as PipelineData>::DataStubType)] {
		&self.outputs
	}
}

impl PipelineNode<UFOData> for Constant {
	fn get_info(&self) -> &dyn NodeInfo<UFOData> {
		self
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		_target_port: usize,
		_input_data: UFOData,
	) -> Result<(), RunNodeError> {
		unreachable!("Constant nodes do not take input.")
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), RunNodeError>,
	) -> Result<PipelineNodeState, RunNodeError> {
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
