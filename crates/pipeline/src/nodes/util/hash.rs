use std::sync::Arc;

use sha2::{Digest, Sha256};
use ufo_util::data::PipelineData;

use crate::{errors::PipelineError, nodes::PipelineNode};

// TODO: hash datatype
// TODO: select hash method
#[derive(Clone)]
pub struct Hash {}

impl Hash {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for Hash {
	fn default() -> Self {
		Self::new()
	}
}

impl PipelineNode for Hash {
	fn run<F>(&self, send_data: F, input: Vec<PipelineData>) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let data = match input.first().unwrap() {
			PipelineData::Binary { data, .. } => data,
			_ => panic!("bad data type"),
		};

		let mut hasher = Sha256::new();
		hasher.update(&**data);
		let result = hasher.finalize();

		send_data(0, PipelineData::Text(Arc::new(format!("{:X}", result))))?;

		return Ok(());
	}
}
