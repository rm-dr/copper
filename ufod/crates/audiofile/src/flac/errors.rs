//! FLAC parsing errors
use crate::common::{picturetype::PictureTypeError, vorbiscomment::VorbisCommentError};
use std::{error::Error, fmt::Display, string::FromUtf8Error};

// TODO: refactor errors?

#[allow(missing_docs)]
#[derive(Debug)]
pub enum FlacError {
	// TODO: multiple comment blocks are an error
	/// FLAC does not start with 0x66 0x4C 0x61 0x43
	BadMagicBytes,

	/// The first metablock isn't StreamInfo
	BadFirstBlock,

	/// We got an invalid metadata block type
	BadMetablockType(u8),

	/// We encountered an i/o error while processing
	IoError(std::io::Error),

	/// We could not parse a vorbis comment
	VorbisComment(VorbisCommentError),

	/// We tried to decode a string, but found invalid UTF-8
	FailedStringDecode(FromUtf8Error),

	/// We tried to read a block, but it was out of spec.
	MalformedBlock,

	/// We tried to decode a bad picture type
	PictureTypeError(PictureTypeError),
}

impl Display for FlacError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading flac"),
			Self::BadMagicBytes => write!(f, "flac signature is missing or malformed"),
			Self::BadFirstBlock => write!(f, "first metablock isn't streaminfo"),
			Self::BadMetablockType(x) => write!(f, "invalid flac metablock type `{x}`"),
			Self::VorbisComment(_) => write!(f, "error while decoding vorbis comment"),
			Self::FailedStringDecode(_) => write!(f, "error while decoding string"),
			Self::MalformedBlock => write!(f, "malformed flac block"),
			Self::PictureTypeError(_) => write!(f, "bad picture type"),
		}
	}
}

impl Error for FlacError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			Self::IoError(e) => e,
			Self::VorbisComment(e) => e,
			Self::FailedStringDecode(e) => e,
			Self::PictureTypeError(e) => e,
			_ => return None,
		})
	}
}

impl From<std::io::Error> for FlacError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<VorbisCommentError> for FlacError {
	fn from(value: VorbisCommentError) -> Self {
		Self::VorbisComment(value)
	}
}

impl From<FromUtf8Error> for FlacError {
	fn from(value: FromUtf8Error) -> Self {
		Self::FailedStringDecode(value)
	}
}

impl From<PictureTypeError> for FlacError {
	fn from(value: PictureTypeError) -> Self {
		Self::PictureTypeError(value)
	}
}
