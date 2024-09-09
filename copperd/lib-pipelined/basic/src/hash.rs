use copper_util::HashType;
use pipelined_node_base::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PipelinePortID,
		ProcessSignalError, RunNodeError,
	},
	data::CopperData,
	helpers::DataSource,
};
use sha2::{Digest, Sha256, Sha512};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{Cursor, Read},
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
		let data = match self {
			Self::MD5 { context } => context.compute().to_vec(),
			Self::SHA256 { hasher } => hasher.finalize().to_vec(),
			Self::SHA512 { hasher } => hasher.finalize().to_vec(),
		};

		CopperData::Hash {
			hash_type: format,
			data,
		}
	}
}

pub struct Hash {
	/// None if disconnected, `Uninitialized` if unset
	data: Option<DataSource>,
	hasher: Option<HashComputer>,
}

// Inputs: "data", Bytes
// Outputs: "hash", Hash
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
			data: None,
			hasher: Some(HashComputer::new(hash_type)),
		})
	}
}

impl Node<CopperData> for Hash {
	fn process_signal(&mut self, signal: NodeSignal<CopperData>) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { port } => match port.id().as_str() {
				"data" => {
					if self.data.is_some() {
						unreachable!("tried to connect an input twice")
					}
					self.data = Some(DataSource::Uninitialized)
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::DisconnectInput { port } => match port.id().as_str() {
				"data" => {
					if self.data.is_none() {
						unreachable!("tried to disconnect an input that hasn't been connected")
					}

					if matches!(self.data, Some(DataSource::Uninitialized)) {
						return Err(ProcessSignalError::RequiredInputEmpty);
					}
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::ReceiveInput { port, data } => {
				if self.data.is_none() {
					unreachable!("received input to a disconnected port")
				}

				match port.id().as_str() {
					"data" => match data {
						CopperData::Blob { source, mime } => {
							self.data.as_mut().unwrap().consume(mime, source);
						}

						_ => return Err(ProcessSignalError::InputWithBadType),
					},

					_ => return Err(ProcessSignalError::InputPortDoesntExist),
				}
			}
		}

		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		match self.data.as_mut() {
			None => return Err(RunNodeError::RequiredInputNotConnected),

			Some(DataSource::Uninitialized) => {
				return Ok(NodeState::Pending("input not ready"));
			}

			Some(DataSource::Url { data, .. }) => {
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

			Some(DataSource::Binary { data, is_done, .. }) => {
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
