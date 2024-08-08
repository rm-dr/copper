use lofty::{
	config::ParseOptions,
	file::AudioFile,
	tag::{Accessor, Tag},
};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::HashMap,
	io::{Cursor, Read, Seek},
};

use crate::{
	model::{AudioItemType, ItemType},
	pipeline::{PipelineData, PipelineDataType, PipelineError},
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
	fn get_inputs() -> &'static [(&'static str, PipelineDataType)] {
		&[("data", PipelineDataType::Binary)]
	}

	fn get_outputs() -> &'static [(&'static str, PipelineDataType)] {
		&[
			("title", PipelineDataType::Text),
			("album", PipelineDataType::Text),
			("artist", PipelineDataType::Text),
			("genre", PipelineDataType::Text),
			("comment", PipelineDataType::Text),
			("track", PipelineDataType::Text),
			("disk", PipelineDataType::Text),
			("disk_total", PipelineDataType::Text),
			("year", PipelineDataType::Text),
		]
	}

	fn run(
		inputs: HashMap<SmartString<LazyCompact>, PipelineData>,
	) -> Result<HashMap<SmartString<LazyCompact>, PipelineData>, PipelineError> {
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
				("title".into(), title),
				("album".into(), album),
				("artist".into(), artist),
				("genre".into(), genre),
				("comment".into(), comment),
				("track".into(), track),
				("disk".into(), disk),
				("disk_total".into(), disk_total),
				("year".into(), year),
			])
		} else {
			HashMap::from([
				("title".into(), PipelineData::None),
				("album".into(), PipelineData::None),
				("artist".into(), PipelineData::None),
				("genre".into(), PipelineData::None),
				("comment".into(), PipelineData::None),
				("track".into(), PipelineData::None),
				("disk".into(), PipelineData::None),
				("disk_total".into(), PipelineData::None),
				("year".into(), PipelineData::None),
			])
		};

		return Ok(h);
	}
}
