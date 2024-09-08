use sha2::{Digest, Sha256, Sha512};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{Cursor, Read},
	sync::Arc,
};

use crate::{
	base::{
		InitNodeError, Node, NodeInfo, NodeParameterValue, NodeState, PipelineData, PipelinePortID,
		RunNodeError,
	},
	data::{CopperData, CopperDataStub, HashType},
	helpers::DataSource,
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

	fn finish(self) -> CopperData {
		let format = self.hash_type();
		let v = match self {
			Self::MD5 { context } => context.compute().to_vec(),
			Self::SHA256 { hasher } => hasher.finalize().to_vec(),
			Self::SHA512 { hasher } => hasher.finalize().to_vec(),
		};

		CopperData::Hash {
			hash_type: format,
			data: Arc::new(v),
		}
	}
}

pub struct Hash {
	inputs: BTreeMap<PipelinePortID, CopperDataStub>,
	outputs: BTreeMap<PipelinePortID, CopperDataStub>,

	data: DataSource,
	hasher: Option<HashComputer>,
}

impl Hash {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 1 });
		}

		let hash_type: HashType = if let Some(value) = params.get("hash_type") {
			match value {
				NodeParameterValue::String(hash_type) => {
					serde_json::from_str(&format!("\"{hash_type}\"")).unwrap()
				}
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "hash_type".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "value".into(),
			});
		};

		Ok(Self {
			inputs: BTreeMap::from([(PipelinePortID::new("data"), CopperDataStub::Bytes)]),

			outputs: BTreeMap::from([(
				PipelinePortID::new("hash"),
				CopperDataStub::Hash { hash_type },
			)]),

			data: DataSource::Uninitialized,
			hasher: Some(HashComputer::new(hash_type)),
		})
	}
}

impl NodeInfo<CopperData> for Hash {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.outputs
	}
}

impl Node<CopperData> for Hash {
	fn get_info(&self) -> &dyn NodeInfo<CopperData> {
		self
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: CopperData,
	) -> Result<(), RunNodeError> {
		match target_port.id().as_str() {
			"data" => match input_data {
				CopperData::Bytes { source, mime } => {
					self.data.consume(mime, source);
				}

				_ => unreachable!("Received input with unexpected type"),
			},

			_ => unreachable!("Received input on invalid port {target_port}"),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(NodeState::Pending("args not ready"));
			}

			DataSource::Url { data, .. } => {
				self.hasher
					.as_mut()
					.unwrap()
					.update(&mut Cursor::new(&**data))?;

				send_data(
					PipelinePortID::new("hash"),
					self.hasher.take().unwrap().finish(),
				)?;
				return Ok(NodeState::Done);
			}

			DataSource::Binary { data, is_done, .. } => {
				while let Some(data) = data.pop_front() {
					self.hasher
						.as_mut()
						.unwrap()
						.update(&mut Cursor::new(&*data))?
				}

				if *is_done {
					send_data(
						PipelinePortID::new("hash"),
						self.hasher.take().unwrap().finish(),
					)?;
					return Ok(NodeState::Done);
				} else {
					return Ok(NodeState::Pending("waiting for data"));
				}
			}
		};
	}
}
