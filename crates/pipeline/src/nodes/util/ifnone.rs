use ufo_util::data::PipelineData;

use crate::{errors::PipelineError, nodes::PipelineNode};

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
	fn run<F>(&self, send_data: F, input: Vec<PipelineData>) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let d = input.first().unwrap();
		let ifnone = input.get(1).unwrap();

		send_data(
			0,
			match *d {
				PipelineData::None(_) => ifnone.clone(),
				_ => d.clone(),
			},
		)?;

		return Ok(());
	}
}
