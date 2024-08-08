//! Decode and write Vorbis comment blocks

use smartstring::{LazyCompact, SmartString};
use std::{
	fmt::Display,
	io::{Cursor, Read, Write},
	string::FromUtf8Error,
};

use super::tagtype::TagType;

#[derive(Debug)]
#[allow(missing_docs)]
pub enum VorbisCommentError {
	/// We encountered an IoError while processing a block
	IoError(std::io::Error),

	/// We tried to decode a string, but got invalid data
	FailedStringDecode(FromUtf8Error),

	/// The given comment string isn't within spec
	MalformedCommentString(String),

	/// The comment we're reading is invalid
	MalformedData,
}

impl Display for VorbisCommentError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading vorbis comments"),
			Self::FailedStringDecode(_) => {
				write!(f, "string decode error while reading vorbis comments")
			}
			Self::MalformedCommentString(x) => {
				write!(f, "malformed comment string `{x}`")
			}

			Self::MalformedData => {
				write!(f, "malformed comment data")
			}
		}
	}
}

impl std::error::Error for VorbisCommentError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::IoError(x) => Some(x),
			Self::FailedStringDecode(x) => Some(x),
			_ => None,
		}
	}
}

impl From<std::io::Error> for VorbisCommentError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<FromUtf8Error> for VorbisCommentError {
	fn from(value: FromUtf8Error) -> Self {
		Self::FailedStringDecode(value)
	}
}

/// A decoded vorbis comment block
#[derive(Debug)]
pub struct VorbisComment {
	/// This comment's vendor string
	pub vendor: SmartString<LazyCompact>,

	/// List of (tag, value)
	pub comments: Vec<(TagType, String)>,
}

impl VorbisComment {
	/// Try to decode the given data as a vorbis comment block
	pub fn decode(data: &[u8]) -> Result<Self, VorbisCommentError> {
		let mut d = Cursor::new(data);

		// This is re-used whenever we need to read four bytes
		let mut block = [0u8; 4];

		let vendor = {
			d.read_exact(&mut block)
				.map_err(|_| VorbisCommentError::MalformedData)?;

			let length = u32::from_le_bytes(block);
			let mut text = vec![0u8; length.try_into().unwrap()];

			d.read_exact(&mut text)
				.map_err(|_| VorbisCommentError::MalformedData)?;

			String::from_utf8(text)?
		};

		d.read_exact(&mut block)
			.map_err(|_| VorbisCommentError::MalformedData)?;
		let n_comments: usize = u32::from_le_bytes(block).try_into().unwrap();

		let mut comments = Vec::with_capacity(n_comments);
		for _ in 0..n_comments {
			let comment = {
				d.read_exact(&mut block)
					.map_err(|_| VorbisCommentError::MalformedData)?;

				let length = u32::from_le_bytes(block);
				let mut text = vec![0u8; length.try_into().unwrap()];

				d.read_exact(&mut text)
					.map_err(|_| VorbisCommentError::MalformedData)?;

				String::from_utf8(text)?
			};
			let (var, val) = comment
				.split_once('=')
				.ok_or(VorbisCommentError::MalformedCommentString(comment.clone()))?;
			if !val.is_empty() {
				// Make sure empty strings are saved as "None"
				comments.push((
					match &var.to_uppercase()[..] {
						"TITLE" => TagType::TrackTitle,
						"ALBUM" => TagType::Album,
						"TRACKNUMBER" => TagType::TrackNumber,
						"ARTIST" => TagType::TrackArtist,
						"ALBUMARTIST" => TagType::AlbumArtist,
						"GENRE" => TagType::Genre,
						"ISRC" => TagType::Isrc,
						"DATE" => TagType::ReleaseDate,
						"TOTALTRACKS" => TagType::TrackTotal,
						"LYRICS" => TagType::Lyrics,
						x => TagType::Other(x.into()),
					},
					val.into(),
				))
			};
		}

		Ok(Self {
			vendor: vendor.into(),
			comments,
		})
	}
}

impl VorbisComment {
	/// Get the number of bytes that `encode()` will write.
	pub fn get_len(&self) -> usize {
		let mut sum = 4 + self.vendor.len() + 4;

		for (tagtype, value) in &self.comments {
			let tagtype_str = match tagtype {
				TagType::TrackTitle => "TITLE",
				TagType::Album => "ALBUM",
				TagType::TrackNumber => "TRACKNUMBER",
				TagType::TrackArtist => "ARTIST",
				TagType::AlbumArtist => "ALBUMARTIST",
				TagType::Genre => "GENRE",
				TagType::Isrc => "ISRC",
				TagType::ReleaseDate => "DATE",
				TagType::TrackTotal => "TOTALTRACKS",
				TagType::Lyrics => "LYRICS",
				TagType::Comment => "COMMENT",
				TagType::DiskNumber => "DISKNUMBER",
				TagType::DiskTotal => "DISKTOTAL",
				TagType::Year => "YEAR",
				TagType::Other(x) => x,
			}
			.to_uppercase();

			let str = format!("{tagtype_str}={value}");
			sum += 4 + str.len();
		}

		return sum;
	}

	/// Try to encode this vorbis comment
	pub fn encode(&self, target: &mut impl Write) -> Result<(), std::io::Error> {
		target.write_all(&u32::try_from(self.vendor.len()).unwrap().to_le_bytes())?;
		target.write_all(self.vendor.as_bytes())?;

		target.write_all(&u32::try_from(self.comments.len()).unwrap().to_le_bytes())?;

		for (tagtype, value) in &self.comments {
			let tagtype_str = match tagtype {
				TagType::TrackTitle => "TITLE",
				TagType::Album => "ALBUM",
				TagType::TrackNumber => "TRACKNUMBER",
				TagType::TrackArtist => "ARTIST",
				TagType::AlbumArtist => "ALBUMARTIST",
				TagType::Genre => "GENRE",
				TagType::Isrc => "ISRC",
				TagType::ReleaseDate => "DATE",
				TagType::TrackTotal => "TOTALTRACKS",
				TagType::Lyrics => "LYRICS",
				TagType::Comment => "COMMENT",
				TagType::DiskNumber => "DISKNUMBER",
				TagType::DiskTotal => "DISKTOTAL",
				TagType::Year => "YEAR",
				TagType::Other(x) => x,
			}
			.to_uppercase();

			let str = format!("{tagtype_str}={value}");
			target.write_all(&u32::try_from(str.len()).unwrap().to_le_bytes())?;
			target.write_all(str.as_bytes())?;
		}

		return Ok(());
	}
}
