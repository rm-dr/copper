use std::sync::Arc;

use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};

use crate::{data::UFOData, UFOContext};

#[derive(Clone)]
pub struct Print {
	input: Option<UFOData>,
}

impl Print {
	pub fn new() -> Self {
		Self { input: None }
	}
}

impl PipelineNode for Print {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
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
		_ctx: Arc<Self::NodeContext>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, UFOData) -> Result<(), PipelineError>,
	{
		println!("{:?}", self.input);
		Ok(PipelineNodeState::Done)
	}
}
