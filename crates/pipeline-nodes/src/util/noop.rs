use std::sync::Arc;

use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_storage::data::{StorageData, StorageDataStub};

use crate::UFOContext;

#[derive(Clone)]
pub struct Noop {
	inputs: Vec<(PipelinePortLabel, StorageDataStub)>,
}

impl Noop {
	pub fn new(inputs: Vec<(PipelinePortLabel, StorageDataStub)>) -> Self {
		Self { inputs }
	}
}

impl PipelineNode for Noop {
	type NodeContext = UFOContext;
	type DataType = StorageData;

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
