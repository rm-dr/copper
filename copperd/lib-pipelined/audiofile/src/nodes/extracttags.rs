use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use copper_util::mime::MimeType;
use itertools::Itertools;
use pipelined_node_base::{
	data::{CopperData, CopperDataStub},
	helpers::DataSource,
	CopperContext,
};
use pipelined_pipeline::{
	base::{InitNodeError, Node, NodeInfo, NodeState, PipelineData, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Info for a [`ExtractTags`] node
pub struct ExtractTagsInfo {
	inputs: BTreeMap<PipelinePortID, CopperDataStub>,
	outputs: BTreeMap<PipelinePortID, CopperDataStub>,
	tags: BTreeMap<PipelinePortID, TagType>,
}

impl ExtractTagsInfo {
	/// Generate node info from parameters
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
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
			inputs: BTreeMap::from([(PipelinePortID::new("data"), CopperDataStub::Bytes)]),
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
			tags: tags
				.into_iter()
				.unique()
				.map(|x| (PipelinePortID::new(Into::<&str>::into(&x)), x))
				.collect(),
		})
	}
}

impl NodeInfo<CopperData> for ExtractTagsInfo {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
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
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
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

impl Node<CopperData> for ExtractTags {
	fn get_info(&self) -> &dyn NodeInfo<CopperData> {
		&self.info
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: CopperData,
	) -> Result<(), RunNodeError> {
		match target_port.id().as_str() {
			"data" => match input_data {
				CopperData::Bytes { source, mime } => {
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
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
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

			DataSource::Url { data, .. } => {
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
					for (port, tag_type) in self.info.tags.iter() {
						if let Some((_, tag_value)) =
							comment.comment.comments.iter().find(|(t, _)| t == tag_type)
						{
							send_data(
								port.clone(),
								CopperData::Text {
									value: Arc::new(tag_value.clone()),
								},
							)?;
						} else {
							send_data(
								port.clone(),
								CopperData::None {
									data_type: CopperDataStub::Text,
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
