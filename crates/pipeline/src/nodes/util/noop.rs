use ufo_util::data::{PipelineData, PipelineDataType};

use crate::{
	errors::PipelineError,
	nodes::{PipelineNode, PipelineNodeState},
	syntax::labels::PipelinePortLabel,
};

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
	fn init<F>(
		&mut self,
		send_data: F,
		input: Vec<PipelineData>,
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
