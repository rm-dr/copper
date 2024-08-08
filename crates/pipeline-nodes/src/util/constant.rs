use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_storage::data::StorageData;

use crate::UFOContext;

#[derive(Clone)]
pub struct Constant {
	value: StorageData,
}

impl Constant {
	pub fn new(value: StorageData) -> Self {
		Self { value }
	}
}

impl PipelineNode for Constant {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
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
