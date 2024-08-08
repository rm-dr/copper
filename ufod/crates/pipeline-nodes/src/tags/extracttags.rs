use itertools::Itertools;
use std::{io::Read, sync::Arc};
use ufo_audiofile::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeError, PipelineNodeState},
	labels::PipelinePortID,
};
use ufo_util::mime::MimeType;

use crate::{
	data::{UFOData, UFODataStub},
	helpers::DataSource,
	nodetype::{UFONodeType, UFONodeTypeError},
	traits::UFONode,
	UFOContext,
};

// TODO: fail after max buffer size
pub struct ExtractTags {
	tags: Vec<TagType>,
	blob_fragment_size: u64,
	data: DataSource,
	reader: FlacBlockReader,
}

impl ExtractTags {
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext, tags: Vec<TagType>) -> Self {
		Self {
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

impl PipelineNode for ExtractTags {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineNodeError> {
		match port {
			0 => match data {
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

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineNodeError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineNodeError>,
	{
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

impl ExtractTags {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Bytes)]
	}
}

impl UFONode for ExtractTags {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs().len(),
			_ => unreachable!(),
		})
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> Result<bool, UFONodeTypeError> {
		Ok(Self::input_default_type(stub, ctx, input_idx)? == input_type)
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs()
				.iter()
				.enumerate()
				.find(|(_, (n, _))| PipelinePortID::new(n) == *input_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		})
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs().get(input_idx).unwrap().1,
			_ => unreachable!(),
		})
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> Result<usize, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::ExtractTags { tags } => tags.len(),
			_ => unreachable!(),
		})
	}

	fn output_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_idx: usize,
	) -> Result<UFODataStub, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::ExtractTags { tags } => {
				assert!(output_idx < tags.len());
				UFODataStub::Text
			}
			_ => unreachable!(),
		})
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Result<Option<usize>, UFONodeTypeError> {
		Ok(match stub {
			UFONodeType::ExtractTags { tags } => tags
				.iter()
				.enumerate()
				.find(|(_, t)| PipelinePortID::new(Into::<&str>::into(*t)) == *output_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		})
	}
}
