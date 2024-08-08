use sha2::{Digest, Sha256, Sha512};
use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_storage::data::{HashType, StorageData};

use crate::UFOContext;

#[derive(Clone)]
pub struct Hash {
	data: Option<StorageData>,
	hash_type: HashType,
}

impl Hash {
	pub fn new(hash_type: HashType) -> Self {
		Self {
			data: None,
			hash_type,
		}
	}
}

impl PipelineNode for Hash {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
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
		_ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let data = match self.data.as_ref().unwrap() {
			StorageData::Binary { data, .. } => data,
			_ => panic!("bad data type"),
		};

		let result = match self.hash_type {
			HashType::MD5 => md5::compute(&**data).to_vec(),
			HashType::SHA256 => {
				let mut hasher = Sha256::new();
				hasher.update(&**data);
				hasher.finalize().to_vec()
			}
			HashType::SHA512 => {
				let mut hasher = Sha512::new();
				hasher.update(&**data);
				hasher.finalize().to_vec()
			}
		};

		send_data(
			0,
			StorageData::Hash {
				format: self.hash_type,
				data: Arc::new(result),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}
