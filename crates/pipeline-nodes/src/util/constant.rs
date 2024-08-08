use std::sync::Arc;

use ufo_pipeline::{
	data::PipelineData,
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};

use crate::UFOContext;

#[derive(Clone)]
pub struct Constant {
	value: PipelineData,
}

impl Constant {
	pub fn new(value: PipelineData) -> Self {
		Self { value }
	}
}

impl PipelineNode for Constant {
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<UFOContext>,
		input: Vec<PipelineData>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 0);
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
