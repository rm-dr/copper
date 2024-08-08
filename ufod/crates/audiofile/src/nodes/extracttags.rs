use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use itertools::Itertools;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};
use ufo_node_base::{
	data::{UFOData, UFODataStub},
	helpers::DataSource,
	UFOContext,
};
use ufo_pipeline::{
	api::{InitNodeError, NodeInfo, PipelineData, Node, NodeState, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use ufo_util::mime::MimeType;

/// Info for a [`ExtractTags`] node
pub struct ExtractTagsInfo {
	inputs: [(PipelinePortID, UFODataStub); 1],
	outputs: Vec<(PipelinePortID, UFODataStub)>,
	tags: Vec<TagType>,
}

impl ExtractTagsInfo {
	/// Generate node info from parameters
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
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
			inputs: [(PipelinePortID::new("data"), UFODataStub::Bytes)],
			outputs: {
				let mut out = Vec::new();
				for t in &tags {
					out.push((
						PipelinePortID::new(Into::<&str>::into(t)),
						UFODataStub::Text,
					))
				}
				out
			},
			tags: tags.into_iter().unique().collect(),
		})
	}
}

impl NodeInfo<UFOData> for ExtractTagsInfo {
	fn inputs(&self) -> &[(PipelinePortID, <UFOData as PipelineData>::DataStubType)] {
		&self.inputs
	}

	fn outputs(&self) -> &[(PipelinePortID, <UFOData as PipelineData>::DataStubType)] {
		&self.outputs
	}
}

/// Extract tags from audio metadata
pub struct ExtractTags {
	info: ExtractTagsInfo,
	blob_fragment_size: u64,
	data: DataSource,
	reader: FlacBlockReader,
}

impl ExtractTags {
	/// Create a new [`ExtractTags`] node
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, InitNodeError> {
		Ok(Self {
			info: ExtractTagsInfo::new(params)?,
			blob_fragment_size: ctx.blob_fragment_size,
			data: DataSource::Uninitialized,
			reader: FlacBlockReader::new(FlacBlockSelector {
				pick_vorbiscomment: true,
				..Default::default()
			}),
		})
	}
}

impl Node<UFOData> for ExtractTags {
	fn get_info(&self) -> &dyn ufo_pipeline::api::NodeInfo<UFOData> {
		&self.info
	}

	fn take_input(&mut self, target_port: usize, input_data: UFOData) -> Result<(), RunNodeError> {
		match target_port {
			0 => match input_data {
				UFOData::Bytes { source, mime } => {
					if mime != MimeType::Flac {
						return Err(RunNodeError::UnsupportedFormat(format!(
							"cannot read tags from `{}`",
							mime
						)));
					}

					self.data.consume(mime, source);
				}

				_ => unreachable!("Received unexpected data type"),
			},

			_ => unreachable!("Received data at invalid port"),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		// Push latest data into tag reader
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(NodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
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

			DataSource::File { file, .. } => {
				let mut v = Vec::new();
				let n = file
					.by_ref()
					.take(self.blob_fragment_size)
					.read_to_end(&mut v)?;
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
					for (i, tag_type) in self.info.tags.iter().enumerate() {
						if let Some(tag_value) = comment.comment.comments.get(tag_type) {
							send_data(
								i,
								UFOData::Text {
									value: Arc::new(tag_value.clone()),
								},
							)?;
						} else {
							send_data(
								i,
								UFOData::None {
									data_type: UFODataStub::Text,
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
