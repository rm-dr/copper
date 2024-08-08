use lofty::{
	config::ParseOptions,
	file::AudioFile,
	tag::{Accessor, Tag},
};
use std::collections::HashMap;

use super::{Extractor, ExtractorError, ExtractorOutput};
use crate::model::{AudioItemType, ItemReader, ItemType};

pub struct TagExtractor {}

impl TagExtractor {
	fn parse_flac(mut data_read: &mut dyn ItemReader) -> Result<Option<Tag>, ExtractorError> {
		let t = lofty::flac::FlacFile::read_from(&mut data_read, ParseOptions::new());
		if t.is_err() {
			return Err(ExtractorError::FileSystemError(Box::new(t.err().unwrap())));
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

	fn parse_mp3(mut data_read: &mut dyn ItemReader) -> Result<Option<Tag>, ExtractorError> {
		let t = lofty::mpeg::MpegFile::read_from(&mut data_read, ParseOptions::new());
		if t.is_err() {
			return Err(ExtractorError::FileSystemError(Box::new(t.err().unwrap())));
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

impl Extractor for TagExtractor {
	fn supports_type(data_type: ItemType) -> bool {
		match data_type {
			ItemType::Binary => false,
			ItemType::Text => false,
			ItemType::Audio(x) => match x {
				AudioItemType::Flac => true,
				AudioItemType::Mp3 => true,
			},
		}
	}

	fn extract(
		data_type: ItemType,
		data_read: &mut dyn ItemReader,
	) -> Result<ExtractorOutput, ExtractorError> {
		let t = match data_type {
			ItemType::Audio(x) => match x {
				AudioItemType::Flac => Self::parse_flac(data_read),
				AudioItemType::Mp3 => Self::parse_mp3(data_read),
			},
			_ => return Err(ExtractorError::UnsupportedDataType),
		};

		if t.is_err() {
			return Err(t.err().unwrap());
		}
		let t = t.unwrap();

		let h = if let Some(t) = t {
			let title = t
				.title()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let album = t
				.album()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let artist = t
				.artist()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let genre = t
				.genre()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let comment = t
				.comment()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let track = t
				.comment()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let disk = t
				.disk()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let disk_total = t
				.disk_total()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);
			let year = t
				.year()
				.map(|x| ExtractorOutput::Text(x.to_string()))
				.unwrap_or(ExtractorOutput::None);

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
				("title".to_string(), ExtractorOutput::None),
				("album".to_string(), ExtractorOutput::None),
				("artist".to_string(), ExtractorOutput::None),
				("genre".to_string(), ExtractorOutput::None),
				("comment".to_string(), ExtractorOutput::None),
				("track".to_string(), ExtractorOutput::None),
				("disk".to_string(), ExtractorOutput::None),
				("disk_total".to_string(), ExtractorOutput::None),
				("year".to_string(), ExtractorOutput::None),
			])
		};

		return Ok(ExtractorOutput::Multi(h));
	}
}
