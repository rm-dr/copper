use ufo_util::data::PipelineData;

use crate::{
	errors::PipelineError,
	nodes::{PipelineNode, PipelineNodeState},
};

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
	fn init<F>(
		&mut self,
		send_data: F,
		mut input: Vec<PipelineData>,
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
