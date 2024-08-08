use sha2::{Digest, Sha256};
use std::sync::Arc;
use ufo_pipeline::{
	data::PipelineData,
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};

use crate::UFOContext;

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
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		mut input: Vec<PipelineData>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: Arc<Self::RunContext>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
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
