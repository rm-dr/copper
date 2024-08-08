use std::str::FromStr;

use serde_with::DeserializeFromStr;
use smartstring::{LazyCompact, SmartString};

#[derive(Debug, Hash, PartialEq, Eq, Clone, DeserializeFromStr)]
pub enum TagType {
	Other(SmartString<LazyCompact>),
	Album,
	AlbumArtist,
	Comment,
	ReleaseDate,
	DiskNumber,
	DiskTotal,
	Genre,
	Isrc,
	Lyrics,
	TrackNumber, // This track's number in its album
	TrackTotal,  // The total number of tracks in this track's album
	TrackTitle,
	TrackArtist,
	Year,
}

// This is a "user-facing" string.
// File format code should use format-specific strings.
impl<'b, 'a: 'b> From<&'a TagType> for &'b str {
	fn from(value: &'a TagType) -> Self {
		match value {
			// This must match `From<&str>` below
			TagType::Album => "Album",
			TagType::AlbumArtist => "AlbumArtist",
			TagType::Comment => "Comment",
			TagType::ReleaseDate => "ReleaseDate",
			TagType::DiskNumber => "DiskNumber",
			TagType::DiskTotal => "DiskTotal",
			TagType::Genre => "Genre",
			TagType::Isrc => "ISRC",
			TagType::Lyrics => "Lyrics",
			TagType::TrackNumber => "TrackNumber",
			TagType::TrackTotal => "TrackTotal",
			TagType::TrackTitle => "Title",
			TagType::TrackArtist => "Artist",
			TagType::Year => "Year",
			TagType::Other(x) => &x,
		}
	}
}

// TODO: better error
impl From<&str> for TagType {
	fn from(s: &str) -> Self {
		// This must match `From<&_>` above
		match s {
			"Album" => Self::Album,
			"AlbumArtist" => Self::AlbumArtist,
			"Comment" => Self::Comment,
			"ReleaseDate" => Self::ReleaseDate,
			"DiskNumber" => Self::DiskNumber,
			"DiskTotal" => Self::DiskTotal,
			"Genre" => Self::Genre,
			"ISRC" => Self::Isrc,
			"Lyrics" => Self::Lyrics,
			"TrackNumber" => Self::TrackNumber,
			"TrackTotal" => Self::TrackTotal,
			"Title" => Self::TrackTitle,
			"Artist" => Self::TrackArtist,
			"Year" => Self::Year,
			x => Self::Other(x.into()),
		}
	}
}

impl FromStr for TagType {
	type Err = std::convert::Infallible;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::from(s))
	}
}
