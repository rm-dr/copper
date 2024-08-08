use itertools::Itertools;
use std::{
	io::{Cursor, Read, Seek},
	sync::Arc,
};
use ufo_audiofile::{
	common::{tagtype::TagType, vorbiscomment::VorbisComment},
	flac::flac_read_tags,
};
use ufo_util::data::{AudioFormat, BinaryFormat, PipelineData, PipelineDataType};

use crate::{errors::PipelineError, PipelineStatelessRunner};

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

impl PipelineStatelessRunner for ExtractTags {
	fn run(&self, data: Vec<Arc<PipelineData>>) -> Result<Vec<Arc<PipelineData>>, PipelineError> {
		let data = data.first().unwrap();

		let (data_type, data) = match data.as_ref() {
			PipelineData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		let mut data_read = Cursor::new(data);
		let tagger = match data_type {
			BinaryFormat::Audio(x) => match x {
				AudioFormat::Flac => Self::parse_flac(&mut data_read),
				AudioFormat::Mp3 => unimplemented!(),
			},
			_ => return Err(PipelineError::UnsupportedDataType),
		};
		if let Err(e) = tagger {
			return Err(e);
		}
		let tagger = tagger.unwrap();

		let mut out = Vec::with_capacity(self.tags.len());
		for tag_type in &self.tags {
			if let Some(tag_value) = tagger.get_tag(tag_type) {
				out.push(Arc::new(PipelineData::Text(tag_value)))
			} else {
				out.push(Arc::new(PipelineData::None(PipelineDataType::Text)))
			}
		}

		return Ok(out);
	}
}
