use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::PipeData,
	helpers::DataSource,
	CopperContext,
};
use copper_storaged::AttrDataStub;
use copper_util::mime::MimeType;
use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read};

/// Extract tags from audio metadata
pub struct ExtractTags {
	tags: BTreeMap<PortName, TagType>,

	blob_fragment_size: u64,
	reader: FlacBlockReader,

	/// None if disconnected, `Uninitialized` if unset
	data: Option<DataSource>,
}

impl ExtractTags {
	/// Create a new [`ExtractTags`] node
	pub fn new(
		ctx: &CopperContext,
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

			blob_fragment_size: ctx.blob_fragment_size,
			data: None,
			reader: FlacBlockReader::new(FlacBlockSelector {
				pick_vorbiscomment: true,
				..Default::default()
			}),
		})
	}
}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
impl Node<PipeData> for ExtractTags {
	fn process_signal(&mut self, signal: NodeSignal<PipeData>) -> Result<(), ProcessSignalError> {
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

			NodeSignal::ReceiveInput { port, data } => match port.id().as_str() {
				"data" => match data {
					PipeData::Blob { source, mime } => {
						if mime != MimeType::Flac {
							return Err(ProcessSignalError::UnsupportedFormat(format!(
								"cannot read tags from `{}`",
								mime
							)));
						}

						self.data.as_mut().unwrap().consume(mime, source);
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
		send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		// Push latest data into tag reader
		match self.data.as_mut() {
			None => return Err(RunNodeError::RequiredInputNotConnected),

			Some(DataSource::Uninitialized) => {
				return Ok(NodeState::Pending("waiting for data"));
			}

			Some(DataSource::Binary { data, is_done, .. }) => {
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

			Some(DataSource::Url { data, .. }) => {
				let mut v = Vec::new();
				let n = data.take(self.blob_fragment_size).read_to_end(&mut v)?;
				self.reader
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				if n == 0 {
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
