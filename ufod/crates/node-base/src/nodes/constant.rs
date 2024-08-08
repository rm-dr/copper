use std::collections::BTreeMap;

use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::{
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineNode, PipelineNodeError,
		PipelineNodeState,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

use crate::{data::UFOData, UFOContext};
pub struct Constant {
	outputs: [NodeOutputInfo<<UFOData as PipelineData>::DataStubType>; 1],
	value: UFOData,
}

impl Constant {
	pub fn new(
		_ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Self {
		if params.len() != 1 {
			panic!()
		}

		let value = if let Some(value) = params.get("value") {
			match value {
				NodeParameterValue::Data(data) => data.clone(),
				_ => panic!(),
			}
		} else {
			panic!()
		};

		Self {
			outputs: [NodeOutputInfo {
				name: PipelinePortID::new("out"),
				produces_type: value.as_stub(),
			}],

			value,
		}
	}
}

impl PipelineNode<UFOData> for Constant {
	fn inputs(&self) -> &[NodeInputInfo<<UFOData as PipelineData>::DataStubType>] {
		&[]
	}

	fn outputs(&self) -> &[NodeOutputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.outputs
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		_target_port: usize,
		_input_data: UFOData,
	) -> Result<(), PipelineNodeError> {
		unreachable!()
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
