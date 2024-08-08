use itertools::Itertools;
use std::{
	io::{Cursor, Read, Seek},
	sync::Arc,
};
use ufo_audiofile::{
	common::{tagtype::TagType, vorbiscomment::VorbisComment},
	flac::flac_read_tags,
};
use ufo_util::{
	data::{PipelineData, PipelineDataType},
	mime::MimeType,
};

use crate::{
	errors::PipelineError,
	nodes::{PipelineNode, PipelineNodeState},
};

#[derive(Clone)]
pub struct ExtractTags {
	data: Option<PipelineData>,
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
	fn init<F>(
		&mut self,
		_send_data: F,
		mut input: Vec<PipelineData>,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let (data_type, data) = match self.data.as_ref().unwrap() {
			PipelineData::Binary {
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
				send_data(i, PipelineData::Text(Arc::new(tag_value)))?;
			} else {
				send_data(i, PipelineData::None(PipelineDataType::Text))?;
			}
		}

		return Ok(PipelineNodeState::Done);
	}
}
