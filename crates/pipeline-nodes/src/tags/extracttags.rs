use itertools::Itertools;
use std::{
	io::{Cursor, Read, Seek},
	sync::Arc,
};
use ufo_audiofile::{
	common::{tagtype::TagType, vorbiscomment::VorbisComment},
	flac::flac_read_tags,
};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_util::mime::MimeType;

use crate::{helpers::UFONode, nodetype::UFONodeType, UFOContext};

#[derive(Clone)]
pub struct ExtractTags {
	data: Option<MetaDbData>,
	tags: Vec<TagType>,
}

impl ExtractTags {
	pub fn new(tags: Vec<TagType>) -> Self {
		Self {
			data: None,
			tags: tags.into_iter().unique().collect(),
		}
	}
}

impl ExtractTags {
	fn parse_flac<R>(read: R) -> Result<VorbisComment, PipelineError>
	where
		R: Read + Seek,
	{
		let tags = flac_read_tags(read).unwrap();
		return Ok(tags.unwrap());
	}
}

impl PipelineNode for ExtractTags {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		mut input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let (data_type, data) = match self.data.as_ref().unwrap() {
			MetaDbData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		let mut data_read = Cursor::new(&**data);
		let tagger = match data_type {
			MimeType::Flac => Self::parse_flac(&mut data_read),
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		}?;

		for (i, tag_type) in self.tags.iter().enumerate() {
			if let Some(tag_value) = tagger.get_tag(tag_type) {
				send_data(i, MetaDbData::Text(Arc::new(tag_value)))?;
			} else {
				send_data(i, MetaDbData::None(MetaDbDataStub::Text))?;
			}
		}

		return Ok(PipelineNodeState::Done);
	}
}

impl ExtractTags {
	fn inputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("data", MetaDbDataStub::Binary)]
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
