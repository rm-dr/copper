use std::sync::Arc;

use sha2::{Digest, Sha256};
use ufo_util::data::PipelineData;

use crate::{
	errors::PipelineError,
	nodes::{PipelineNode, PipelineNodeState},
};

// TODO: hash datatype
// TODO: select hash method
#[derive(Clone)]
pub struct Hash {
	data: Option<PipelineData>,
}

impl Hash {
	pub fn new() -> Self {
		Self { data: None }
	}
}

impl Default for Hash {
	fn default() -> Self {
		Self::new()
	}
}

impl PipelineNode for Hash {
	fn init<F>(
		&mut self,
		_send_data: F,
		mut input: Vec<PipelineData>,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let data = match self.data.as_ref().unwrap() {
			PipelineData::Binary { data, .. } => data,
			_ => panic!("bad data type"),
		};

		let mut hasher = Sha256::new();
		hasher.update(&**data);
		let result = hasher.finalize();

		send_data(0, PipelineData::Text(Arc::new(format!("{:X}", result))))?;

		return Ok(PipelineNodeState::Done);
	}
}
