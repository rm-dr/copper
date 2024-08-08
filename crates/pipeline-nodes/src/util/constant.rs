use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};

use crate::{helpers::UFONode, nodetype::UFONodeType, UFOContext};

#[derive(Clone)]
pub struct Constant {
	value: MetaDbData,
}

impl Constant {
	pub fn new(value: MetaDbData) -> Self {
		Self { value }
	}
}

impl PipelineNode for Constant {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		input: Vec<Self::DataType>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.is_empty());
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
		_input_type: MetaDbDataStub,
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
	) -> MetaDbDataStub {
		unreachable!()
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Constant { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
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
