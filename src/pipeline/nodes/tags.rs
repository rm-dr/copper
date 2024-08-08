use lofty::{
	config::ParseOptions,
	file::AudioFile,
	tag::{Accessor, Tag},
};
use std::{
	collections::HashMap,
	io::{Cursor, Read, Seek},
};

use crate::{
	model::{AudioItemType, ItemType},
	pipeline::{PipelineData, PipelineError},
};

use super::PipelineNode;

pub struct ExtractTag {}

impl ExtractTag {
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

impl PipelineNode for ExtractTag {
	fn run(
		inputs: HashMap<String, PipelineData>,
	) -> Result<HashMap<String, PipelineData>, PipelineError> {
		let data = inputs.get("data").unwrap();

		let (data_type, data) = match data {
			PipelineData::Binary { data_type, data } => (data_type, data),
			_ => panic!(),
		};

		let mut data_read = Cursor::new(data);
		let t = match data_type {
			ItemType::Audio(x) => match x {
				AudioItemType::Flac => Self::parse_flac(&mut data_read),
				AudioItemType::Mp3 => Self::parse_mp3(&mut data_read),
			},
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		if t.is_err() {
			return Err(t.err().unwrap());
		}
		let t = t.unwrap();

		let h = if let Some(t) = t {
			let title = t
				.title()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let album = t
				.album()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let artist = t
				.artist()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let genre = t
				.genre()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let comment = t
				.comment()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let track = t
				.comment()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let disk = t
				.disk()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let disk_total = t
				.disk_total()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);
			let year = t
				.year()
				.map(|x| PipelineData::Text(x.to_string()))
				.unwrap_or(PipelineData::None);

			HashMap::from([
				("title".to_string(), title),
				("album".to_string(), album),
				("artist".to_string(), artist),
				("genre".to_string(), genre),
				("comment".to_string(), comment),
				("track".to_string(), track),
				("disk".to_string(), disk),
				("disk_total".to_string(), disk_total),
				("year".to_string(), year),
			])
		} else {
			HashMap::from([
				("title".to_string(), PipelineData::None),
				("album".to_string(), PipelineData::None),
				("artist".to_string(), PipelineData::None),
				("genre".to_string(), PipelineData::None),
				("comment".to_string(), PipelineData::None),
				("track".to_string(), PipelineData::None),
				("disk".to_string(), PipelineData::None),
				("disk_total".to_string(), PipelineData::None),
				("year".to_string(), PipelineData::None),
			])
		};

		return Ok(h);
	}
}
