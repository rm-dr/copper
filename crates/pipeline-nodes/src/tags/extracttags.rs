use itertools::Itertools;
use std::{
	io::{Seek, SeekFrom},
	sync::Arc,
};
use ufo_audiofile::{common::tagtype::TagType, flac::flac_read_tags};
use ufo_database::metadb::data::MetaDbDataStub;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};
use ufo_util::mime::MimeType;

use crate::{
	data::UFOData, errors::PipelineError, helpers::ArcVecBuffer, nodetype::UFONodeType,
	traits::UFONode, UFOContext,
};

// TODO: fail after max buffer size
pub struct ExtractTags {
	tags: Vec<TagType>,
	format: Option<MimeType>,
	fragments: ArcVecBuffer,
	is_done: bool,
}

impl ExtractTags {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext, tags: Vec<TagType>) -> Self {
		Self {
			tags: tags.into_iter().unique().collect(),
			fragments: ArcVecBuffer::new(),
			is_done: false,
			format: None,
		}
	}
}

impl PipelineNode for ExtractTags {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => {
				let (format, fragment, is_last) = match data {
					UFOData::Blob {
						format,
						fragment,
						is_last,
					} => (format, fragment, is_last),
					_ => panic!(),
				};

				assert!(!self.is_done);

				if let Some(f) = &self.format {
					assert!(*f == format);
				} else {
					self.format = Some(format);
				}

				self.fragments.push_back(fragment);
				self.is_done = is_last;
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.format.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		self.fragments.seek(SeekFrom::Start(0))?;
		let tagger = match self.format.as_ref().unwrap() {
			MimeType::Flac => {
				let r = flac_read_tags(&mut self.fragments);
				if r.is_err() {
					return Ok(PipelineNodeState::Pending("malformed block"));
				}
				r.unwrap()
			}
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		if tagger.is_none() {
			return Ok(PipelineNodeState::Pending("no comment block found"));
		}
		let tagger = tagger.unwrap();

		for (i, tag_type) in self.tags.iter().enumerate() {
			if let Some(tag_value) = tagger.get_tag(tag_type) {
				send_data(i, UFOData::Text(Arc::new(tag_value)))?;
			} else {
				send_data(i, UFOData::None(MetaDbDataStub::Text))?;
			}
		}

		return Ok(PipelineNodeState::Done);
	}
}

impl ExtractTags {
	fn inputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("data", MetaDbDataStub::Blob)]
	}
}

impl UFONode for ExtractTags {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs().len(),
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: MetaDbDataStub,
	) -> bool {
		Self::input_default_type(stub, ctx, input_idx) == input_type
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs()
				.iter()
				.enumerate()
				.find(|(_, (n, _))| PipelinePortLabel::from(*n) == *input_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs().get(input_idx).unwrap().1,
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::ExtractTags { tags } => tags.len(),
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::ExtractTags { tags } => {
				assert!(output_idx < tags.len());
				MetaDbDataStub::Text
			}
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::ExtractTags { tags } => tags
				.iter()
				.enumerate()
				.find(|(_, t)| PipelinePortLabel::from(Into::<&str>::into(*t)) == *output_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}
}
