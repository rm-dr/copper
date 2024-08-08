use std::{error::Error, fmt::Display, string::FromUtf8Error};

use crate::common::{picturetype::PictureTypeError, vorbiscomment::VorbisCommentError};

// TODO: simplify errors?

#[derive(Debug)]
pub enum FlacError {
	// TODO: multiple comment blocks are an error
	IoError(std::io::Error),
	FailedStringDecode(FromUtf8Error),
	BadPictureType(PictureTypeError),
	VorbisCommentError(VorbisCommentError),
	BadMagicBytes, // FLAC does not start with 0x66 0x4C 0x61 0x43
}

impl Display for FlacError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading flac"),
			Self::FailedStringDecode(_) => write!(f, "failed string decode while reading flac"),
			Self::BadPictureType(_) => write!(f, "flac has invalid picture type"),
			Self::VorbisCommentError(_) => write!(f, "error while decoding metadata block"),
			Self::BadMagicBytes => write!(f, "flac signature is missing"),
		}
	}
}

impl Error for FlacError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			Self::IoError(e) => e,
			Self::FailedStringDecode(e) => e,
			Self::BadPictureType(e) => e,
			Self::VorbisCommentError(e) => e,
			_ => return None,
		})
	}
}

impl From<std::io::Error> for FlacError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<FromUtf8Error> for FlacError {
	fn from(value: FromUtf8Error) -> Self {
		Self::FailedStringDecode(value)
	}
}

impl From<PictureTypeError> for FlacError {
	fn from(value: PictureTypeError) -> Self {
		Self::BadPictureType(value)
	}
}

impl From<VorbisCommentError> for FlacError {
	fn from(value: VorbisCommentError) -> Self {
		Self::VorbisCommentError(value)
	}
}
