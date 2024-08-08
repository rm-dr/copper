use sha2::{Digest, Sha256, Sha512};
use std::{collections::VecDeque, sync::Arc};
use ufo_db_metastore::data::HashType;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	nodetype::UFONodeType,
	traits::UFONode,
	UFOContext,
};

enum HashComputer {
	MD5 { context: md5::Context },
	SHA256 { hasher: Sha256 },
	SHA512 { hasher: Sha512 },
}

impl HashComputer {
	fn new(hash_type: HashType) -> Self {
		match hash_type {
			HashType::MD5 => Self::MD5 {
				context: md5::Context::new(),
			},
			HashType::SHA256 => Self::SHA256 {
				hasher: Sha256::new(),
			},
			HashType::SHA512 => Self::SHA512 {
				hasher: Sha512::new(),
			},
		}
	}

	fn update(&mut self, buf: &[u8]) {
		match self {
			Self::MD5 { context } => {
				context.consume(buf);
			}
			Self::SHA256 { hasher } => {
				hasher.update(buf);
			}
			Self::SHA512 { hasher } => {
				hasher.update(buf);
			}
		}
	}

	fn hash_type(&self) -> HashType {
		match self {
			Self::MD5 { .. } => HashType::MD5,
			Self::SHA256 { .. } => HashType::SHA256,
			Self::SHA512 { .. } => HashType::SHA512,
		}
	}

	fn finish(self) -> UFOData {
		let format = self.hash_type();
		let v = match self {
			Self::MD5 { context } => context.compute().to_vec(),
			Self::SHA256 { hasher } => hasher.finalize().to_vec(),
			Self::SHA512 { hasher } => hasher.finalize().to_vec(),
		};

		UFOData::Hash {
			format,
			data: Arc::new(v),
		}
	}
}

enum HashData {
	Blob {
		fragments: VecDeque<Arc<Vec<u8>>>,
		is_done: bool,
	},
	Binary(Arc<Vec<u8>>),
}

pub struct Hash {
	data: Option<HashData>,
	hasher: Option<HashComputer>,
}

impl Hash {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext, hash_type: HashType) -> Self {
		Self {
			data: None,
			hasher: Some(HashComputer::new(hash_type)),
		}
	}
}

impl PipelineNode for Hash {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => match data {
				UFOData::Binary { data, .. } => {
					self.data = Some(HashData::Binary(data));
				}
				UFOData::Blob {
					fragment, is_last, ..
				} => match &mut self.data {
					None => {
						self.data = Some(HashData::Blob {
							fragments: VecDeque::from([fragment]),
							is_done: is_last,
						})
					}
					Some(HashData::Blob { fragments, is_done }) => {
						assert!(!*is_done);
						fragments.push_back(fragment);
						*is_done = is_last;
					}
					Some(_) => panic!(),
				},
				_ => todo!(),
			},
			_ => unreachable!("bad input port {port}"),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.data.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		match self.data.as_mut().unwrap() {
			HashData::Binary(data) => {
				self.hasher.as_mut().unwrap().update(&**data);
				send_data(0, self.hasher.take().unwrap().finish())?;
				return Ok(PipelineNodeState::Done);
			}
			HashData::Blob { fragments, is_done } => {
				while let Some(data) = fragments.pop_front() {
					self.hasher.as_mut().unwrap().update(&**data)
				}
				if *is_done {
					send_data(0, self.hasher.take().unwrap().finish())?;
					return Ok(PipelineNodeState::Done);
				} else {
					return Ok(PipelineNodeState::Pending("waiting for data"));
				}
			}
		};
	}
}

impl UFONode for Hash {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> bool {
		match stub {
			UFONodeType::Hash { .. } => {
				assert!(input_idx < 1);
				match input_type {
					UFODataStub::Blob | UFODataStub::Binary => true,
					_ => false,
				}
			}
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Hash { .. } => match Into::<&str>::into(input_name) {
				"data" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(stub: &UFONodeType, _ctx: &UFOContext, input_idx: usize) -> UFODataStub {
		match stub {
			UFONodeType::Hash { .. } => {
				assert!(input_idx < 1);
				UFODataStub::Binary
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> UFODataStub {
		match stub {
			UFONodeType::Hash { hash_type } => {
				assert!(output_idx == 0);
				UFODataStub::Hash {
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
