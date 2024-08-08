use lofty::{
	config::ParseOptions,
	file::AudioFile,
	tag::{Accessor, Tag},
};
use std::{
	io::{Cursor, Read, Seek},
	sync::Arc,
};
use ufo_util::data::{AudioFormat, BinaryFormat, PipelineData};

use crate::{errors::PipelineError, PipelineStatelessRunner};

pub struct ExtractTags {}

impl ExtractTags {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for ExtractTags {
	fn default() -> Self {
		Self::new()
	}
}

impl ExtractTags {
	fn parse_flac<R>(mut data_read: &mut R) -> Result<Option<Tag>, PipelineError>
	where
		R: Read + Seek,
	{
		let t = lofty::flac::FlacFile::read_from(&mut data_read, ParseOptions::new());
		if t.is_err() {
			return Err(PipelineError::FileSystemError(Box::new(t.err().unwrap())));
		}
		let t = t.unwrap();

		#[allow(clippy::manual_map)]
		Ok(if let Some(vorbis) = t.vorbis_comments() {
			Some(Tag::from(vorbis.clone()))
		} else if let Some(id3v2) = t.id3v2() {
			// id3v2 Discouraged by spec
			Some(Tag::from(id3v2.clone()))
		} else {
			None
		})
	}

	fn parse_mp3<R>(mut data_read: &mut R) -> Result<Option<Tag>, PipelineError>
	where
		R: Read + Seek,
	{
		let t = lofty::mpeg::MpegFile::read_from(&mut data_read, ParseOptions::new());
		if t.is_err() {
			return Err(PipelineError::FileSystemError(Box::new(t.err().unwrap())));
		}
		let t = t.unwrap();

		#[allow(clippy::manual_map)]
		Ok(if let Some(id3v2) = t.id3v2() {
			Some(Tag::from(id3v2.clone()))
		} else {
			None
		})
	}
}

impl PipelineStatelessRunner for ExtractTags {
	fn run(
		&self,
		data: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError> {
		let data = data.first().unwrap();

		let (data_type, data) = match data.as_ref().unwrap().as_ref() {
			PipelineData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		let mut data_read = Cursor::new(data);
		let t = match data_type {
			BinaryFormat::Audio(x) => match x {
				AudioFormat::Flac => Self::parse_flac(&mut data_read),
				AudioFormat::Mp3 => Self::parse_mp3(&mut data_read),
			},
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		if t.is_err() {
			return Err(t.err().unwrap());
		}
		let t = t.unwrap();

		let h = if let Some(t) = t {
			let title = t.title().map(|x| PipelineData::Text(x.to_string()));
			let album = t.album().map(|x| PipelineData::Text(x.to_string()));
			let artist = t.artist().map(|x| PipelineData::Text(x.to_string()));
			let genre = t.genre().map(|x| PipelineData::Text(x.to_string()));
			let comment = t.comment().map(|x| PipelineData::Text(x.to_string()));
			let track = t.comment().map(|x| PipelineData::Text(x.to_string()));
			let disk = t.disk().map(|x| PipelineData::Text(x.to_string()));
			let disk_total = t.disk_total().map(|x| PipelineData::Text(x.to_string()));
			let year = t.year().map(|x| PipelineData::Text(x.to_string()));

			vec![
				title.map(Arc::new),
				album.map(Arc::new),
				artist.map(Arc::new),
				genre.map(Arc::new),
				comment.map(Arc::new),
				track.map(Arc::new),
				disk.map(Arc::new),
				disk_total.map(Arc::new),
				year.map(Arc::new),
			]
		} else {
			vec![None, None, None, None, None, None, None, None, None]
		};

		return Ok(h);
	}
}
