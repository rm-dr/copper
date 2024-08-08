use sha2::{Digest, Sha256};
use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};

use crate::{
	data::{HashType, UFOData},
	UFOContext,
};

// TODO: hash datatype
// TODO: select hash method
#[derive(Clone)]
pub struct Hash {
	data: Option<UFOData>,
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
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		mut input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let data = match self.data.as_ref().unwrap() {
			UFOData::Binary { data, .. } => data,
			_ => panic!("bad data type"),
		};

		let mut hasher = Sha256::new();
		hasher.update(&**data);
		let result = hasher.finalize();

		send_data(
			0,
			UFOData::Hash {
				format: HashType::SHA256,
				data: Arc::new(result.to_vec()),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}
