use lofty::{
	config::ParseOptions,
	file::AudioFile,
	tag::{Accessor, Tag},
};
use std::{
	collections::HashMap,
	io::{Cursor, Read, Seek},
};

use crate::pipeline::{
	components::PipelinePortLabel,
	data::{AudioFormat, BinaryFormat, PipelineData, PipelineDataType},
	errors::PipelineError,
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
	fn get_input(input: &PipelinePortLabel) -> Option<PipelineDataType> {
		match AsRef::as_ref(input) {
			"data" => Some(PipelineDataType::Binary),
			_ => None,
		}
	}

	fn get_output(input: &PipelinePortLabel) -> Option<PipelineDataType> {
		match AsRef::as_ref(input) {
			"title" => Some(PipelineDataType::Text),
			"album" => Some(PipelineDataType::Text),
			"artist" => Some(PipelineDataType::Text),
			"genre" => Some(PipelineDataType::Text),
			"comment" => Some(PipelineDataType::Text),
			"track" => Some(PipelineDataType::Text),
			"disk" => Some(PipelineDataType::Text),
			"disk_total" => Some(PipelineDataType::Text),
			"year" => Some(PipelineDataType::Text),
			_ => None,
		}
	}

	fn get_inputs() -> impl Iterator<Item = PipelinePortLabel> {
		["data"].iter().map(|x| (*x).into())
	}

	fn get_outputs() -> impl Iterator<Item = PipelinePortLabel> {
		[
			"title",
			"album",
			"artist",
			"genre",
			"comment",
			"track",
			"disk",
			"disk_total",
			"year",
		]
		.iter()
		.map(|x| (*x).into())
	}

	fn run(
		inputs: HashMap<PipelinePortLabel, Option<PipelineData>>,
	) -> Result<HashMap<PipelinePortLabel, Option<PipelineData>>, PipelineError> {
		let data = inputs.get(&"data".into()).unwrap();

		let (data_type, data) = match data.as_ref().unwrap() {
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
				("title".into(), None),
				("album".into(), None),
				("artist".into(), None),
				("genre".into(), None),
				("comment".into(), None),
				("track".into(), None),
				("disk".into(), None),
				("disk_total".into(), None),
				("year".into(), None),
			])
		};

		return Ok(h);
	}
}
