use std::sync::Arc;

use ufo_pipeline::{
	data::PipelineData,
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};

use crate::UFOContext;

#[derive(Clone)]
pub struct Print {
	input: Option<PipelineData>,
}

impl Print {
	pub fn new() -> Self {
		Self { input: None }
	}
}

impl PipelineNode for Print {
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		mut input: Vec<PipelineData>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.input = input.pop();
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		println!("{:?}", self.input);
		Ok(PipelineNodeState::Done)
	}
}
