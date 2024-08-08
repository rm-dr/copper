use ufo_util::data::{PipelineData, PipelineDataType};

use crate::{errors::PipelineError, nodes::PipelineNode, syntax::labels::PipelinePortLabel};

#[derive(Clone)]
pub struct Noop {
	_inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
}

impl Noop {
	pub fn new(inputs: Vec<(PipelinePortLabel, PipelineDataType)>) -> Self {
		Self { _inputs: inputs }
	}
}

impl PipelineNode for Noop {
	fn run<F>(&self, send_data: F, input: Vec<PipelineData>) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		for (i, v) in input.into_iter().enumerate() {
			send_data(i, v)?;
		}

		return Ok(());
	}
}
