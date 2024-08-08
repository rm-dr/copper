use sha2::{Digest, Sha256, Sha512};
use std::{
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_ds_core::data::HashType;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeError, PipelineNodeState},
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	helpers::DataSource,
	nodetype::{UFONodeType, UFONodeTypeError},
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

	fn update(&mut self, data: &mut dyn Read) -> Result<(), std::io::Error> {
		match self {
			Self::MD5 { context } => {
				std::io::copy(data, context)?;
			}
			Self::SHA256 { hasher } => {
				std::io::copy(data, hasher)?;
			}
			Self::SHA512 { hasher } => {
				std::io::copy(data, hasher)?;
			}
		}

		return Ok(());
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

pub struct Hash {
	data: DataSource,
	hasher: Option<HashComputer>,
}

impl Hash {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext, hash_type: HashType) -> Self {
		Self {
			data: DataSource::Uninitialized,
			hasher: Some(HashComputer::new(hash_type)),
		}
	}
}

impl PipelineNode for Hash {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineNodeError> {
		match port {
			0 => match data {
				UFOData::Bytes { source, mime } => {
					self.data.consume(mime, source);
				}

				_ => panic!("bad input type"),
			},

			_ => unreachable!("bad input port {port}"),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineNodeError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineNodeError>,
	{
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(PipelineNodeState::Pending("args not ready"));
			}

			DataSource::File { file, .. } => {
				self.hasher.as_mut().unwrap().update(file)?;
				send_data(0, self.hasher.take().unwrap().finish())?;
				return Ok(PipelineNodeState::Done);
			}

			DataSource::Binary { data, is_done, .. } => {
				while let Some(data) = data.pop_front() {
					self.hasher
						.as_mut()
						.unwrap()
						.update(&mut Cursor::new(&*data))?
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
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		})
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> Result<bool, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { .. } => {
				assert!(input_idx < 1);
				matches!(input_type, UFODataStub::Bytes)
			}
			_ => unreachable!(),
		})
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { .. } => match input_name.id().as_str() {
				"data" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		})
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { .. } => {
				assert!(input_idx < 1);
				UFODataStub::Bytes
			}
			_ => unreachable!(),
		})
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		})
	}

	fn output_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { hash_type } => {
				assert!(output_idx == 0);
				UFODataStub::Hash {
					hash_type: *hash_type,
				}
			}
			_ => unreachable!(),
		})
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::Hash { .. } => {
				if output_name.id().as_str() == "hash" {
					Some(0)
				} else {
					None
				}
			}
			_ => unreachable!(),
		})
	}
}
