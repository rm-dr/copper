use std::sync::Arc;

use ufo_pipeline::{
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
	syntax::labels::PipelinePortLabel,
};

use crate::UFOContext;

#[derive(Clone)]
pub struct Noop {
	inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
}

impl Noop {
	pub fn new(inputs: Vec<(PipelinePortLabel, PipelineDataType)>) -> Self {
		Self { inputs }
	}
}

impl PipelineNode for Noop {
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		input: Vec<PipelineData>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == self.inputs.len());
		for (i, v) in input.into_iter().enumerate() {
			send_data(i, v)?;
		}
		Ok(PipelineNodeState::Done)
	}
}
