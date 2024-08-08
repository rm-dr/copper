use sha2::{Digest, Sha256, Sha512};
use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_storage::data::{HashType, StorageData, StorageDataStub};

use crate::{nodetype::UFONodeType, UFOContext, UFONode};

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

impl Hash {
	fn inputs() -> &'static [(&'static str, StorageDataStub)] {
		&[("data", StorageDataStub::Binary)]
	}
}

impl UFONode for Hash {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Hash { .. } => Self::inputs().len(),
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: StorageDataStub,
	) -> bool {
		Self::input_default_type(stub, ctx, input_idx) == input_type
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Hash { .. } => Self::inputs()
				.iter()
				.enumerate()
				.find(|(_, (n, _))| PipelinePortLabel::from(*n) == *input_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> StorageDataStub {
		match stub {
			UFONodeType::Hash { .. } => Self::inputs().get(input_idx).unwrap().1,
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> StorageDataStub {
		match stub {
			UFONodeType::Hash { hash_type } => {
				assert!(output_idx == 0);
				StorageDataStub::Hash {
					hash_type: *hash_type,
				}
			}
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Hash { .. } => {
				if Into::<&str>::into(output_name) == "hash" {
					Some(0)
				} else {
					None
				}
			}
			_ => unreachable!(),
		}
	}
}
