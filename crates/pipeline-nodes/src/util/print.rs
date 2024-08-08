use ufo_database::metastore::data::MetastoreDataStub;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
};

#[derive(Clone)]
pub struct Print {
	has_received: bool,
}

impl Print {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			has_received: false,
		}
	}
}

impl PipelineNode for Print {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		assert!(port == 0);
		println!("{data:?}");
		self.has_received = true;
		return Ok(());
	}

	fn run<F>(&mut self, _send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, UFOData) -> Result<(), PipelineError>,
	{
		if self.has_received {
			Ok(PipelineNodeState::Done)
		} else {
			Ok(PipelineNodeState::Pending("args not ready"))
		}
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
		_input_type: MetastoreDataStub,
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
	) -> MetastoreDataStub {
		match stub {
			UFONodeType::Print => {
				assert!(input_idx == 0);
				MetastoreDataStub::Text
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

	fn output_type(_stub: &UFONodeType, _ctx: &UFOContext, _output_idx: usize) -> MetastoreDataStub {
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
