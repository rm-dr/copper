use itertools::Itertools;
use lofty::{
	config::ParseOptions,
	file::AudioFile,
	tag::{ItemKey, Tag},
};
use serde_with::DeserializeFromStr;
use std::{
	fmt::Display,
	io::{Cursor, Read, Seek},
	str::FromStr,
	sync::Arc,
};
use ufo_util::data::{AudioFormat, BinaryFormat, PipelineData, PipelineDataType};

use crate::{errors::PipelineError, PipelineStatelessRunner};

#[derive(Debug, DeserializeFromStr, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TagType {
	Album,
	AlbumArtist,
	Comment,
	ReleaseDate,
	DiskNumber,
	DiskTotal,
	Genre,
	ISRC,
	Lyrics,
	TrackNumber,
	TrackTitle,
	TrackArtist,
	Year,
}

impl TagType {
	pub fn get_type(&self) -> PipelineDataType {
		match self {
			Self::Album
			| Self::AlbumArtist
			| Self::Comment
			| Self::Genre
			| Self::ISRC
			| Self::Lyrics
			| Self::TrackTitle
			| Self::TrackArtist => PipelineDataType::Text,
			Self::ReleaseDate => PipelineDataType::Text,
			Self::DiskNumber => PipelineDataType::Text,
			Self::DiskTotal => PipelineDataType::Text,
			Self::TrackNumber => PipelineDataType::Text,
			Self::Year => PipelineDataType::Text,
		}
	}

	fn extract(&self, t: &Tag) -> PipelineData {
		t.get_string(&match self {
			Self::Album => ItemKey::AlbumTitle,
			Self::AlbumArtist => ItemKey::AlbumArtist,
			Self::Comment => ItemKey::Comment,
			Self::ReleaseDate => ItemKey::ReleaseDate,
			Self::DiskNumber => ItemKey::DiscNumber,
			Self::DiskTotal => ItemKey::DiscNumber,
			Self::Genre => ItemKey::Genre,
			Self::ISRC => ItemKey::Isrc,
			Self::Lyrics => ItemKey::Lyrics,
			Self::TrackNumber => ItemKey::TrackNumber,
			Self::TrackTitle => ItemKey::TrackTitle,
			Self::TrackArtist => ItemKey::TrackArtist,
			Self::Year => ItemKey::Year,
		})
		.map(|x| PipelineData::Text(x.to_string()))
		.unwrap_or(PipelineData::None(PipelineDataType::Text))
	}
}

impl Display for TagType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			// This should match `FromStr` below
			Self::Album => write!(f, "Album"),
			Self::AlbumArtist => write!(f, "AlbumArtist"),
			Self::Comment => write!(f, "Comment"),
			Self::ReleaseDate => write!(f, "ReleaseDate"),
			Self::DiskNumber => write!(f, "DiskNumber"),
			Self::DiskTotal => write!(f, "DiskTotal"),
			Self::Genre => write!(f, "Genre"),
			Self::ISRC => write!(f, "ISRC"),
			Self::Lyrics => write!(f, "Lyrics"),
			Self::TrackNumber => write!(f, "TrackNumber"),
			Self::TrackTitle => write!(f, "Title"),
			Self::TrackArtist => write!(f, "Artist"),
			Self::Year => write!(f, "Year"),
		}
	}
}

// TODO: better error
impl FromStr for TagType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// This should match `Display` above
		Ok(match s {
			"Album" => Self::Album,
			"AlbumArtist" => Self::AlbumArtist,
			"Comment" => Self::Comment,
			"ReleaseDate" => Self::ReleaseDate,
			"DiskNumber" => Self::DiskNumber,
			"DiskTotal" => Self::DiskTotal,
			"Genre" => Self::Genre,
			"ISRC" => Self::ISRC,
			"Lyrics" => Self::Lyrics,
			"TrackNumber" => Self::TrackNumber,
			"Title" => Self::TrackTitle,
			"Artist" => Self::TrackArtist,
			"Year" => Self::Year,
			x => return Err(format!("Unknown tag {x}")),
		})
	}
}

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

/*
impl Default for ExtractTags {
	fn default() -> Self {
		Self::new()
	}
}
*/

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
		let tag = t.unwrap();

		let mut out = Vec::with_capacity(self.tags.len());

		if let Some(tag) = tag {
			for t in &self.tags {
				out.push(Arc::new(t.extract(&tag)))
			}
		} else {
			for t in &self.tags {
				out.push(Arc::new(PipelineData::None(t.get_type())))
			}
		};

		return Ok(out);
	}
}
