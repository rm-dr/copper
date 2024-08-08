use std::sync::Arc;

use ufo_pipeline::{
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
	syntax::labels::PipelinePortLabel,
};

use crate::{
	data::{UFOData, UFODataStub},
	UFOContext,
};

#[derive(Clone)]
pub struct Noop {
	inputs: Vec<(PipelinePortLabel, UFODataStub)>,
}

impl Noop {
	pub fn new(inputs: Vec<(PipelinePortLabel, UFODataStub)>) -> Self {
		Self { inputs }
	}
}

impl PipelineNode for Noop {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		input: Vec<Self::DataType>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == self.inputs.len());
		for (i, v) in input.into_iter().enumerate() {
			send_data(i, v)?;
		}
		Ok(PipelineNodeState::Done)
	}
}
