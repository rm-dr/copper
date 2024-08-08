use sha2::{Digest, Sha256, Sha512};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_ds_core::data::HashType;
use ufo_pipeline::{
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineNode, PipelineNodeError,
		PipelineNodeState,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	helpers::DataSource,
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
			hash_type: format,
			data: Arc::new(v),
		}
	}
}

pub struct Hash {
	inputs: Vec<NodeInputInfo<<UFOData as PipelineData>::DataStubType>>,
	outputs: Vec<NodeOutputInfo<<UFOData as PipelineData>::DataStubType>>,

	data: DataSource,
	hasher: Option<HashComputer>,
}

impl Hash {
	pub fn new(
		_ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Self {
		if params.len() != 1 {
			panic!()
		}

		let hash_type: HashType = if let Some(value) = params.get("value") {
			match value {
				NodeParameterValue::String(hash_type) => serde_json::from_str(hash_type).unwrap(),
				_ => panic!(),
			}
		} else {
			panic!()
		};

		Self {
			inputs: vec![NodeInputInfo {
				name: PipelinePortID::new("data"),
				accepts_type: UFODataStub::Bytes,
			}],

			outputs: vec![NodeOutputInfo {
				name: PipelinePortID::new("hash"),
				produces_type: UFODataStub::Hash {
					hash_type: hash_type,
				},
			}],

			data: DataSource::Uninitialized,
			hasher: Some(HashComputer::new(hash_type)),
		}
	}
}

impl PipelineNode<UFOData> for Hash {
	fn inputs(&self) -> &[NodeInputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.inputs
	}

	fn outputs(&self) -> &[NodeOutputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.outputs
	}

	fn take_input(
		&mut self,
		target_port: usize,
		input_data: UFOData,
	) -> Result<(), PipelineNodeError> {
		match target_port {
			0 => match input_data {
				UFOData::Bytes { source, mime } => {
					self.data.consume(mime, source);
				}

				_ => panic!("bad input type"),
			},

			_ => unreachable!("bad input port {target_port}"),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
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
