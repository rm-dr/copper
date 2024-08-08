use ufo_db_metastore::data::MetastoreDataStub;
use ufo_pipeline::{
	api::{PipelineData, PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
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
		_input_type: MetastoreDataStub,
	) -> bool {
		match stub {
			UFONodeType::Constant { .. } => false,
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		_input_name: &PipelinePortLabel,
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
	) -> MetastoreDataStub {
		unreachable!()
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Constant { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetastoreDataStub {
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
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::ExtractTags { .. } => {
				if Into::<&str>::into(output_name) == "value" {
					Some(0)
				} else {
					None
				}
			}
			_ => unreachable!(),
		}
	}
}
