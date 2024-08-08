use std::sync::Arc;

use ufo_pipeline::{
	data::PipelineData,
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};

use crate::UFOContext;

#[derive(Clone)]
pub struct IfNone {}

impl IfNone {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for IfNone {
	fn default() -> Self {
		Self::new()
	}
}

impl PipelineNode for IfNone {
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		mut input: Vec<PipelineData>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 2);
		let ifnone = input.pop().unwrap();
		let input = input.pop().unwrap();

		send_data(
			0,
			match input {
				PipelineData::None(_) => ifnone,
				_ => input,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}
