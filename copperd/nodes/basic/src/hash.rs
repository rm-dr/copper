use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::{BytesSource, PipeData},
	helpers::{BytesSourceArrayReader, ConnectedInput, OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_util::HashType;
use futures::executor::block_on;
use sha2::{Digest, Sha256, Sha512};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{BufReader, Cursor, Read},
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

	fn finish(self) -> PipeData {
		let format = self.hash_type();
		let data = match self {
			Self::MD5 { context } => context.compute().to_vec(),
			Self::SHA256 { hasher } => hasher.finalize().to_vec(),
			Self::SHA512 { hasher } => hasher.finalize().to_vec(),
		};

		PipeData::Hash {
			hash_type: format,
			data,
		}
	}
}

pub struct Hash {
	data: ConnectedInput<OpenBytesSourceReader>,
	hasher: Option<HashComputer>,
}

// Inputs: "data", Bytes
// Outputs: "hash", Hash
impl Hash {
	pub fn new(
		_ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
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
			data: ConnectedInput::NotConnected,
			hasher: Some(HashComputer::new(hash_type)),
		})
	}
}

impl Node<PipeData, CopperContext> for Hash {
	fn process_signal(
		&mut self,
		ctx: &CopperContext,
		signal: NodeSignal<PipeData>,
	) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { port } => match port.id().as_str() {
				"data" => self.data.connect(),
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::DisconnectInput { port } => match port.id().as_str() {
				"data" => {
					if !self.data.is_connected() {
						unreachable!("disconnected an input that hasn't been connected")
					}
					if !self.data.is_set() {
						return Err(ProcessSignalError::RequiredInputEmpty);
					}
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::ReceiveInput { port, data } => match port.id().as_str() {
				"data" => match data {
					PipeData::Blob { source, mime } => match source {
						BytesSource::Array { .. } => {
							self.data.set(OpenBytesSourceReader::Array(
								BytesSourceArrayReader::new(Some(mime), source).unwrap(),
							));
						}

						BytesSource::S3 { key } => {
							self.data
								.set(OpenBytesSourceReader::S3(block_on(S3Reader::new(
									ctx.objectstore_client.clone(),
									&ctx.objectstore_bucket,
									key,
								))))
						}
					},

					_ => return Err(ProcessSignalError::InputWithBadType),
				},

				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},
		}

		return Ok(());
	}

	fn run(
		&mut self,
		ctx: &CopperContext,
		send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		if !self.data.is_connected() {
			return Err(RunNodeError::RequiredInputNotConnected);
		}

		if !self.data.is_set() {
			return Ok(NodeState::Pending("args not ready"));
		}

		match self.data.value_mut().unwrap() {
			OpenBytesSourceReader::Array(BytesSourceArrayReader { data, is_done, .. }) => {
				while let Some(data) = data.pop_front() {
					self.hasher
						.as_mut()
						.unwrap()
						.update(&mut Cursor::new(&*data))?
				}

				if *is_done {
					send_data(PortName::new("hash"), self.hasher.take().unwrap().finish())?;
					return Ok(NodeState::Done);
				} else {
					return Ok(NodeState::Pending("waiting for data"));
				}
			}

			OpenBytesSourceReader::S3(r) => {
				let mut r = BufReader::new(r);
				self.hasher.as_mut().unwrap().update(&mut r).unwrap();
				send_data(PortName::new("hash"), self.hasher.take().unwrap().finish())?;
				return Ok(NodeState::Done);
			}
		};
	}
}
