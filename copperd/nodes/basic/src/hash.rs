use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError},
	data::{BytesSource, PipeData},
	helpers::{OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_util::HashType;
use sha2::{Digest, Sha256, Sha512};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::BTreeMap,
	io::{Cursor, Read},
};
use tokio::sync::broadcast;

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

pub struct Hash {}

// Inputs: "data", Bytes
// Outputs: "hash", Hash
#[async_trait]
impl Node<PipeData, CopperContext> for Hash {
	async fn run(
		&self,
		ctx: &CopperContext,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, NodeOutput<PipeData>>,
	) -> Result<BTreeMap<PortName, NodeOutput<PipeData>>, RunNodeError> {
		//
		// Extract parameters
		//
		let hash_type: HashType = if let Some(value) = params.remove("hash_type") {
			match value {
				NodeParameterValue::String(hash_type) => {
					serde_json::from_str(&format!("\"{hash_type}\"")).unwrap()
				}
				_ => {
					return Err(RunNodeError::BadParameterType {
						parameter: "hash_type".into(),
					})
				}
			}
		} else {
			return Err(RunNodeError::MissingParameter {
				parameter: "hash_type".into(),
			});
		};
		if let Some((param, _)) = params.first_key_value() {
			return Err(RunNodeError::UnexpectedParameter {
				parameter: param.clone(),
			});
		}

		//
		// Extract inputs
		//
		let data = input.remove(&PortName::new("data"));
		if data.is_none() {
			return Err(RunNodeError::MissingInput {
				port: PortName::new("data"),
			});
		}

		let data = match data.unwrap().get_value().await? {
			None => {
				return Err(RunNodeError::RequiredInputNull {
					port: PortName::new("data"),
				})
			}

			Some(PipeData::Blob { source, .. }) => match source {
				BytesSource::Stream { receiver, .. } => OpenBytesSourceReader::Array(receiver),
				BytesSource::S3 { key } => OpenBytesSourceReader::S3(
					S3Reader::new(ctx.objectstore_client.clone(), &ctx.objectstore_bucket, key)
						.await,
				),
			},

			_ => {
				return Err(RunNodeError::BadInputType {
					port: PortName::new("data"),
				})
			}
		};
		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}

		//
		// Compute hash
		//
		let mut hasher = HashComputer::new(hash_type);
		let mut out = BTreeMap::new();

		match data {
			OpenBytesSourceReader::Array(mut receiver) => {
				loop {
					let rec = receiver.recv().await;
					match rec {
						Ok(d) => {
							hasher = tokio::task::spawn_blocking(move || {
								hasher.update(&mut Cursor::new(&*d)).unwrap();
								hasher // Take and return ownership of `hasher`
							})
							.await?
						}

						Err(broadcast::error::RecvError::Lagged(_)) => {
							return Err(RunNodeError::StreamReceiverLagged)
						}

						Err(broadcast::error::RecvError::Closed) => {
							break;
						}
					}
				}
			}

			OpenBytesSourceReader::S3(mut r) => {
				let mut buf = [0u8; 1_000_000];
				loop {
					let l = r.read(&mut buf).await.unwrap();
					if l != 0 {
						break;
					} else {
						hasher = tokio::task::spawn_blocking(move || {
							hasher
								.update(&mut Cursor::new(&buf).take(l.try_into().unwrap()))
								.unwrap();
							hasher
						})
						.await?;
					}
				}
			}
		};

		out.insert(
			PortName::new("hash"),
			NodeOutput::Plain(Some(hasher.finish())),
		);
		return Ok(out);
	}
}
