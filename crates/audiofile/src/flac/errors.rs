use std::{error::Error, fmt::Display};

use crate::common::vorbiscomment::VorbisCommentError;

// TODO: simplify errors?

#[derive(Debug)]
pub enum FlacError {
	// TODO: multiple comment blocks are an error
	BadMagicBytes, // FLAC does not start with 0x66 0x4C 0x61 0x43
	BadMetablockType(u8),

	IoError(std::io::Error),
	VorbisComment(VorbisCommentError),
}

impl Display for FlacError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading flac"),
			Self::BadMagicBytes => write!(f, "flac signature is missing or malformed"),
			Self::BadMetablockType(x) => write!(f, "invalid flac metablock type `{x}`"),
			Self::VorbisComment(_) => write!(f, "error while decoding vorbis comment"),
		}
	}
}

impl Error for FlacError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			Self::IoError(e) => e,
			Self::VorbisComment(e) => e,
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
