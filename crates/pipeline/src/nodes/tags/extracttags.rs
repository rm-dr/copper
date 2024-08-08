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

use crate::{errors::PipelineError, PipelineNode};

#[derive(Clone)]
pub struct ExtractTags {
	tags: Vec<TagType>,
}

impl ExtractTags {
	pub fn new(tags: Vec<TagType>) -> Self {
		Self {
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
	fn run<F>(&self, send_data: F, input: Vec<PipelineData>) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let data = input.first().unwrap();

		let (data_type, data) = match data {
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

		return Ok(());
	}
}
