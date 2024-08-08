use ufo_pipeline::{
	api::{PipelineData, PipelineNode, PipelineNodeState},
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	nodetype::UFONodeType,
	traits::UFONode,
	UFOContext,
};

#[derive(Clone)]
pub struct Constant {
	value: UFOData,
}

impl Constant {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext, value: UFOData) -> Self {
		Self { value }
	}
}

impl PipelineNode for Constant {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(&mut self, (_port, _data): (usize, UFOData)) -> Result<(), PipelineError> {
		unreachable!()
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}

impl UFONode for Constant {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Constant { .. } => 0,
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		_input_idx: usize,
		_input_type: UFODataStub,
	) -> bool {
		match stub {
			UFONodeType::Constant { .. } => false,
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		_input_name: &PipelinePortID,
	) -> Option<usize> {
		match stub {
			UFONodeType::Constant { .. } => None,
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		_stub: &UFONodeType,
		_ctx: &UFOContext,
		_input_idx: usize,
	) -> UFODataStub {
		unreachable!()
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Constant { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> UFODataStub {
		match stub {
			UFONodeType::Constant { value } => {
				assert!(output_idx == 0);
				value.as_stub()
			}
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Option<usize> {
		match stub {
			UFONodeType::ExtractTags { .. } => {
				if output_name.id().as_str() == "value" {
					Some(0)
				} else {
					None
				}
			}
			_ => unreachable!(),
		}
	}
}
