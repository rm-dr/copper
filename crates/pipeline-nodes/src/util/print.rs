use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};

use crate::{helpers::UFONode, nodetype::UFONodeType, UFOContext};

#[derive(Clone)]
pub struct Print {
	input: Option<MetaDbData>,
}

impl Print {
	pub fn new() -> Self {
		Self { input: None }
	}
}

impl PipelineNode for Print {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		mut input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.input = input.pop();
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, MetaDbData) -> Result<(), PipelineError>,
	{
		println!("{:?}", self.input);
		Ok(PipelineNodeState::Done)
	}
}

impl UFONode for Print {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Print => 1,
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
		_input_type: MetaDbDataStub,
	) -> bool {
		match stub {
			UFONodeType::Print => {
				assert!(input_idx == 0);
				true
			}
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Print => match Into::<&str>::into(input_name) {
				"data" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::Print => {
				assert!(input_idx == 0);
				MetaDbDataStub::Text
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Print => 0,
			_ => unreachable!(),
		}
	}

	fn output_type(_stub: &UFONodeType, _ctx: &UFOContext, _output_idx: usize) -> MetaDbDataStub {
		unreachable!()
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		_output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Print => None,
			_ => unreachable!(),
		}
	}
}
