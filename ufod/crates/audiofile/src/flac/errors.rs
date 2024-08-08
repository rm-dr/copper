//! FLAC errors
use crate::common::{
	picturetype::PictureTypeError,
	vorbiscomment::{VorbisCommentDecodeError, VorbisCommentEncodeError},
};
use std::{error::Error, fmt::Display, string::FromUtf8Error};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum FlacDecodeError {
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
	VorbisComment(VorbisCommentDecodeError),

	/// We tried to decode a string, but found invalid UTF-8
	FailedStringDecode(FromUtf8Error),

	/// We tried to read a block, but it was out of spec.
	MalformedBlock,

	/// We didn't find frame sync bytes where we expected them
	BadSyncBytes,

	/// We tried to decode a bad picture type
	PictureTypeError(PictureTypeError),
}

impl Display for FlacDecodeError {
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
			Self::BadSyncBytes => write!(f, "bad frame sync bytes"),
		}
	}
}

impl Error for FlacDecodeError {
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

impl From<std::io::Error> for FlacDecodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<VorbisCommentDecodeError> for FlacDecodeError {
	fn from(value: VorbisCommentDecodeError) -> Self {
		Self::VorbisComment(value)
	}
}

impl From<FromUtf8Error> for FlacDecodeError {
	fn from(value: FromUtf8Error) -> Self {
		Self::FailedStringDecode(value)
	}
}

impl From<PictureTypeError> for FlacDecodeError {
	fn from(value: PictureTypeError) -> Self {
		Self::PictureTypeError(value)
	}
}

#[allow(missing_docs)]
#[derive(Debug)]
pub enum FlacEncodeError {
	/// We encountered an i/o error while processing
	IoError(std::io::Error),

	/// We could not encode a picture inside a vorbis comment
	VorbisPictureEncodeError,
}

impl Display for FlacEncodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while encoding block"),
			Self::VorbisPictureEncodeError => {
				write!(f, "could not encode picture in vorbis comment")
			}
		}
	}
}

impl Error for FlacEncodeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			Self::IoError(e) => e,
			_ => return None,
		})
	}
}

impl From<std::io::Error> for FlacEncodeError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<VorbisCommentEncodeError> for FlacEncodeError {
	fn from(value: VorbisCommentEncodeError) -> Self {
		match value {
			VorbisCommentEncodeError::IoError(e) => e.into(),
			VorbisCommentEncodeError::PictureEncodeError => Self::VorbisPictureEncodeError,
		}
	}
}
