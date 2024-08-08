use std::sync::Arc;

use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};

use crate::{data::UFOData, UFOContext};

#[derive(Clone)]
pub struct Constant {
	value: UFOData,
}

impl Constant {
	pub fn new(value: UFOData) -> Self {
		Self { value }
	}
}

impl PipelineNode for Constant {
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
		assert!(input.len() == 0);
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
