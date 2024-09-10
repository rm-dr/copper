use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::{BytesSource, PipeData},
	helpers::{BytesSourceArrayReader, ConnectedInput, OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_storaged::AttrDataStub;
use copper_util::MimeType;
use futures::executor::block_on;
use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read};

/// Extract tags from audio metadata
pub struct ExtractTags {
	tags: BTreeMap<PortName, TagType>,
	reader: FlacBlockReader,
	data: ConnectedInput<OpenBytesSourceReader>,
}

impl ExtractTags {
	/// Create a new [`ExtractTags`] node
	pub fn new(
		_ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		let mut tags: Vec<TagType> = Vec::new();
		if let Some(taglist) = params.get("tags") {
			match taglist {
				NodeParameterValue::List(list) => {
					for t in list {
						match t {
							NodeParameterValue::String(s) => tags.push(s.as_str().into()),
							_ => {
								return Err(InitNodeError::BadParameterType {
									param_name: "tags".into(),
								})
							}
						}
					}
				}
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "tags".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "tags".into(),
			});
		}

		Ok(Self {
			/*
			outputs: {
				let mut out = BTreeMap::new();
				for t in &tags {
					out.insert(
						PipelinePortID::new(Into::<&str>::into(t)),
						CopperDataStub::Text,
					);
				}
				out
			},
			*/
			tags: tags
				.into_iter()
				.unique()
				.map(|x| (PortName::new(Into::<&str>::into(&x)), x))
				.collect(),

			data: ConnectedInput::NotConnected,
			reader: FlacBlockReader::new(FlacBlockSelector {
				pick_vorbiscomment: true,
				..Default::default()
			}),
		})
	}
}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
impl Node<PipeData, CopperContext> for ExtractTags {
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
					PipeData::Blob { source, mime } => {
						if mime != MimeType::Flac {
							return Err(ProcessSignalError::UnsupportedFormat(format!(
								"cannot read tags from `{}`",
								mime
							)));
						}

						match source {
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
						}
					}

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
			return Ok(NodeState::Pending("input not ready"));
		}

		match self.data.value_mut().unwrap() {
			OpenBytesSourceReader::Array(BytesSourceArrayReader { data, is_done, .. }) => {
				while let Some(d) = data.pop_front() {
					self.reader
						.push_data(&d)
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.reader
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}

			OpenBytesSourceReader::S3(r) => {
				let mut v = Vec::new();
				r.take(ctx.blob_fragment_size).read_to_end(&mut v).unwrap();
				self.reader
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				if r.is_done() {
					self.reader
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Read and send tags
		if self.reader.has_block() {
			let b = self.reader.pop_block().unwrap();
			match b {
				FlacBlock::VorbisComment(comment) => {
					for (port, tag_type) in self.tags.iter() {
						if let Some((_, tag_value)) =
							comment.comment.comments.iter().find(|(t, _)| t == tag_type)
						{
							send_data(
								port.clone(),
								PipeData::Text {
									value: tag_value.clone(),
								},
							)?;
						} else {
							send_data(
								port.clone(),
								PipeData::None {
									data_type: AttrDataStub::Text,
								},
							)?;
						}
					}
				}
				_ => unreachable!(),
			}

			// We should only have one comment block
			assert!(!self.reader.has_block());
			return Ok(NodeState::Done);
		}

		return Ok(NodeState::Pending("Waiting for data"));
	}
}
