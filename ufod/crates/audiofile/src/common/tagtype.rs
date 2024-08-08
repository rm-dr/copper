//! Cross-format normalized tag types

use serde_with::{DeserializeFromStr, SerializeDisplay};
use smartstring::{LazyCompact, SmartString};
use std::{fmt::Display, str::FromStr};

/// A universal tag type
#[derive(Debug, Hash, PartialEq, Eq, Clone, DeserializeFromStr, SerializeDisplay)]
pub enum TagType {
	/// A tag we didn't recognize
	Other(SmartString<LazyCompact>),

	/// Album name
	Album,
	/// Album artist
	AlbumArtist,
	/// Comment
	Comment,
	/// Release date
	ReleaseDate,
	/// Disk number
	DiskNumber,
	/// Total disks in album
	DiskTotal,
	/// Genre
	Genre,
	/// International standard recording code
	Isrc,
	/// Track lyrics, possibly time-coded
	Lyrics,
	/// This track's number in its album
	TrackNumber,
	/// The total number of tracks in this track's album
	TrackTotal,
	/// The title of this track
	TrackTitle,
	/// This track's artist (the usual `Artist`,
	/// compare to `AlbumArtist`)
	TrackArtist,
	/// The year this track was released
	Year,
}

impl Display for TagType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", Into::<&str>::into(self))
	}
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
			TagType::Other(x) => x,
		}
	}
}

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
