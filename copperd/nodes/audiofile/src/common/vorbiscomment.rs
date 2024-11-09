//! Decode and write Vorbis comment blocks

use base64::Engine;
use smartstring::{LazyCompact, SmartString};
use std::{
	fmt::Display,
	io::{Cursor, Read, Write},
	string::FromUtf8Error,
};

use crate::flac::blocks::{FlacMetablockDecode, FlacMetablockEncode, FlacPictureBlock};

use super::tagtype::TagType;

#[derive(Debug)]
#[allow(missing_docs)]
pub enum VorbisCommentDecodeError {
	/// We encountered an IoError while processing a block
	IoError(std::io::Error),

	/// We tried to decode a string, but got invalid data
	FailedStringDecode(FromUtf8Error),

	/// The given comment string isn't within spec
	MalformedCommentString(String),

	/// The comment we're reading is invalid
	MalformedData,

	/// We tried to decode picture data, but it was malformed.
	MalformedPicture,
}

impl Display for VorbisCommentDecodeError {
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
			Self::MalformedPicture => {
				write!(f, "malformed picture data")
			}
		}
	}
}

impl std::error::Error for VorbisCommentDecodeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::IoError(x) => Some(x),
			Self::FailedStringDecode(x) => Some(x),
			_ => None,
		}
	}
}

impl From<std::io::Error> for VorbisCommentDecodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<FromUtf8Error> for VorbisCommentDecodeError {
	fn from(value: FromUtf8Error) -> Self {
		Self::FailedStringDecode(value)
	}
}

#[derive(Debug)]
#[allow(missing_docs)]
pub enum VorbisCommentEncodeError {
	/// We encountered an IoError while processing a block
	IoError(std::io::Error),

	/// We could not encode picture data
	PictureEncodeError,
}

impl Display for VorbisCommentEncodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading vorbis comments"),
			Self::PictureEncodeError => {
				write!(f, "could not encode picture")
			}
		}
	}
}

impl std::error::Error for VorbisCommentEncodeError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::IoError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<std::io::Error> for VorbisCommentEncodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

/// A decoded vorbis comment block
#[derive(Debug)]
pub struct VorbisComment {
	/// This comment's vendor string
	pub vendor: SmartString<LazyCompact>,

	/// List of (tag, value)
	/// Repeated tags are allowed!
	pub comments: Vec<(TagType, SmartString<LazyCompact>)>,

	/// A list of pictures found in this comment
	pub pictures: Vec<FlacPictureBlock>,
}

impl VorbisComment {
	/// Try to decode the given data as a vorbis comment block
	pub fn decode(data: &[u8]) -> Result<Self, VorbisCommentDecodeError> {
		let mut d = Cursor::new(data);

		// This is re-used whenever we need to read four bytes
		let mut block = [0u8; 4];

		let vendor = {
			#[expect(clippy::map_err_ignore)]
			d.read_exact(&mut block)
				.map_err(|_| VorbisCommentDecodeError::MalformedData)?;

			let length = u32::from_le_bytes(block);
			let mut text = vec![0u8; length.try_into().unwrap()];

			#[expect(clippy::map_err_ignore)]
			d.read_exact(&mut text)
				.map_err(|_| VorbisCommentDecodeError::MalformedData)?;

			String::from_utf8(text)?
		};

		#[expect(clippy::map_err_ignore)]
		d.read_exact(&mut block)
			.map_err(|_| VorbisCommentDecodeError::MalformedData)?;
		let n_comments: usize = u32::from_le_bytes(block).try_into().unwrap();

		let mut comments = Vec::new();
		let mut pictures = Vec::new();
		for _ in 0..n_comments {
			let comment = {
				#[expect(clippy::map_err_ignore)]
				d.read_exact(&mut block)
					.map_err(|_| VorbisCommentDecodeError::MalformedData)?;

				let length = u32::from_le_bytes(block);
				let mut text = vec![0u8; length.try_into().unwrap()];

				#[expect(clippy::map_err_ignore)]
				d.read_exact(&mut text)
					.map_err(|_| VorbisCommentDecodeError::MalformedData)?;

				String::from_utf8(text)?
			};
			let (var, val) =
				comment
					.split_once('=')
					.ok_or(VorbisCommentDecodeError::MalformedCommentString(
						comment.clone(),
					))?;

			if !val.is_empty() {
				if var.to_uppercase() == "METADATA_BLOCK_PICTURE" {
					#[expect(clippy::map_err_ignore)]
					pictures.push(
						FlacPictureBlock::decode(
							&base64::prelude::BASE64_STANDARD
								.decode(val)
								.map_err(|_| VorbisCommentDecodeError::MalformedPicture)?,
						)
						.map_err(|_| VorbisCommentDecodeError::MalformedPicture)?,
					);
				} else {
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
					));
				}
			};
		}

		Ok(Self {
			vendor: vendor.into(),
			comments,
			pictures,
		})
	}
}

impl VorbisComment {
	/// Get the number of bytes that `encode()` will write.
	pub fn get_len(&self) -> u32 {
		let mut sum: u32 = 0;
		sum += u32::try_from(self.vendor.len()).unwrap() + 4;
		sum += 4;

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
			sum += 4 + u32::try_from(str.len()).unwrap();
		}

		for p in &self.pictures {
			// Compute b64 len
			let mut x = p.get_len();
			if x % 3 != 0 {
				x -= x % 3;
				x += 3;
			}

			#[expect(clippy::integer_division)]
			{
				sum += 4 * (x / 3);
			}

			// Add "METADATA_BLOCK_PICTURE="
			sum += 23;

			// Add length bytes
			sum += 4;
		}

		return sum;
	}

	/// Try to encode this vorbis comment
	pub fn encode(&self, target: &mut impl Write) -> Result<(), VorbisCommentEncodeError> {
		target.write_all(&u32::try_from(self.vendor.len()).unwrap().to_le_bytes())?;
		target.write_all(self.vendor.as_bytes())?;

		target.write_all(
			&u32::try_from(self.comments.len() + self.pictures.len())
				.unwrap()
				.to_le_bytes(),
		)?;

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

		for p in &self.pictures {
			let mut pic_data = Vec::new();

			#[expect(clippy::map_err_ignore)]
			p.encode(false, false, &mut pic_data)
				.map_err(|_| VorbisCommentEncodeError::PictureEncodeError)?;

			let pic_string = format!(
				"METADATA_BLOCK_PICTURE={}",
				&base64::prelude::BASE64_STANDARD.encode(&pic_data)
			);

			target.write_all(&u32::try_from(pic_string.len()).unwrap().to_le_bytes())?;
			target.write_all(pic_string.as_bytes())?;
		}

		return Ok(());
	}
}
