use std::sync::Arc;

use ufo_pipeline::{
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};

use crate::{data::UFOData, UFOContext};

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
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		mut input: Vec<Self::DataType>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 2);
		let ifnone = input.pop().unwrap();
		let input = input.pop().unwrap();

		send_data(
			0,
			match input {
				UFOData::None(_) => ifnone,
				_ => input,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}
