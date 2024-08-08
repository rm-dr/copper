use smartstring::{LazyCompact, SmartString};
use std::{fmt::Display, io::Read, string::FromUtf8Error};

use super::tagtype::TagType;

#[derive(Debug)]
pub enum VorbisCommentError {
	IoError(std::io::Error),
	FailedStringDecode(FromUtf8Error),
	MalformedCommentString(String),
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

#[derive(Debug)]
pub struct VorbisComment {
	vendor: SmartString<LazyCompact>,
	comments: Vec<(TagType, String)>,
}

impl VorbisComment {
	pub fn decode<R>(mut read: R) -> Result<Self, VorbisCommentError>
	where
		R: Read,
	{
		// This is re-used whenever we need to read four bytes
		let mut block = [0u8; 4];

		let vendor = {
			read.read_exact(&mut block)?;
			let length = u32::from_le_bytes(block);
			let mut text = vec![0u8; length.try_into().unwrap()];
			read.read_exact(&mut text)?;
			String::from_utf8(text)?
		};

		read.read_exact(&mut block)?;
		let n_comments: usize = u32::from_le_bytes(block).try_into().unwrap();

		let mut comments = Vec::with_capacity(n_comments);
		for _ in 0..n_comments {
			let comment = {
				read.read_exact(&mut block)?;
				let length = u32::from_le_bytes(block);
				let mut text = vec![0u8; length.try_into().unwrap()];
				read.read_exact(&mut text)?;
				String::from_utf8(text)?
			};
			let (var, val) = comment
				.split_once("=")
				.ok_or(VorbisCommentError::MalformedCommentString(comment.clone()))?;
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

		Ok(Self {
			vendor: vendor.into(),
			comments,
		})
	}

	pub fn get_tag(&self, tag: &TagType) -> Option<String> {
		for (t, c) in &self.comments {
			if t == tag {
				// TODO: handle many tags
				return Some(c.clone());
			}
		}
		return None;
	}

	pub fn get_vendor(&self) -> &str {
		&self.vendor
	}
}
