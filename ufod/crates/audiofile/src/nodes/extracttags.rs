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
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineNode, PipelineNodeError,
		PipelineNodeState,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use ufo_util::mime::MimeType;

/// Extract tags from audio metadata
pub struct ExtractTags {
	inputs: Vec<NodeInputInfo<<UFOData as PipelineData>::DataStubType>>,
	outputs: Vec<NodeOutputInfo<<UFOData as PipelineData>::DataStubType>>,

	tags: Vec<TagType>,
	blob_fragment_size: u64,
	data: DataSource,
	reader: FlacBlockReader,
}

impl ExtractTags {
	/// Create a new [`ExtractTags`] node
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Self {
		if params.len() != 1 {
			panic!()
		}

		let mut tags: Vec<TagType> = Vec::new();
		if let Some(taglist) = params.get("tags") {
			match taglist {
				NodeParameterValue::List(list) => {
					for t in list {
						match t {
							NodeParameterValue::String(s) => tags.push(s.as_str().into()),
							_ => panic!(),
						}
					}
				}
				_ => panic!(),
			}
		} else {
			panic!()
		}

		Self {
			inputs: vec![NodeInputInfo {
				name: PipelinePortID::new("data"),
				accepts_type: UFODataStub::Bytes,
			}],

			outputs: {
				let mut out = Vec::new();
				for t in &tags {
					out.push(NodeOutputInfo {
						name: PipelinePortID::new(Into::<&str>::into(t)),
						produces_type: UFODataStub::Text,
					})
				}
				out
			},

			blob_fragment_size: ctx.blob_fragment_size,
			tags: tags.into_iter().unique().collect(),
			data: DataSource::Uninitialized,
			reader: FlacBlockReader::new(FlacBlockSelector {
				pick_vorbiscomment: true,
				..Default::default()
			}),
		}
	}
}

impl PipelineNode<UFOData> for ExtractTags {
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
					if mime != MimeType::Flac {
						return Err(PipelineNodeError::UnsupportedFormat(format!(
							"cannot read tags from `{}`",
							mime
						)));
					}

					self.data.consume(mime, source);
				}

				_ => panic!("bad input type"),
			},

			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
		// Push latest data into tag reader
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(PipelineNodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
				while let Some(d) = data.pop_front() {
					self.reader
						.push_data(&d)
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.reader
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
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
					.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;

				if n == 0 {
					self.reader
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Read and send tags
		if self.reader.has_block() {
			let b = self.reader.pop_block().unwrap();
			match b {
				FlacBlock::VorbisComment(comment) => {
					for (i, tag_type) in self.tags.iter().enumerate() {
						if let Some((_, tag_value)) =
							comment.comment.comments.iter().find(|x| x.0 == *tag_type)
						{
							send_data(i, UFOData::Text(Arc::new(tag_value.clone())))?;
						} else {
							send_data(i, UFOData::None(UFODataStub::Text))?;
						}
					}
				}
				_ => unreachable!(),
			}

			// We should only have one comment block
			assert!(!self.reader.has_block());
			return Ok(PipelineNodeState::Done);
		}

		return Ok(PipelineNodeState::Pending("Waiting for data"));
	}
}
