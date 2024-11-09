//! FLAC errors
use crate::common::{
	picturetype::PictureTypeError,
	vorbiscomment::{VorbisCommentDecodeError, VorbisCommentEncodeError},
};
use std::string::FromUtf8Error;
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum FlacDecodeError {
	/// FLAC does not start with 0x66 0x4C 0x61 0x43
	#[error("flac signature is missing or malformed")]
	BadMagicBytes,

	/// The first metablock isn't StreamInfo
	#[error("first metablock isn't streaminfo")]
	BadFirstBlock,

	/// We got an invalid metadata block type
	#[error("invalid flac metablock type {0}")]
	BadMetablockType(u8),

	/// We encountered an i/o error while processing
	#[error("io error while reading flac")]
	IoError(#[from] std::io::Error),

	/// We could not parse a vorbis comment
	#[error("error while decoding vorbis comment")]
	VorbisComment(#[from] VorbisCommentDecodeError),

	/// We tried to decode a string, but found invalid UTF-8
	#[error("error while decoding string")]
	FailedStringDecode(#[from] FromUtf8Error),

	/// We tried to read a block, but it was out of spec.
	#[error("malformed flac block")]
	MalformedBlock,

	/// We didn't find frame sync bytes where we expected them
	#[error("bad frame sync bytes")]
	BadSyncBytes,

	/// We tried to decode a bad picture type
	#[error("bad picture type")]
	PictureTypeError(#[from] PictureTypeError),
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum FlacEncodeError {
	/// We encountered an i/o error while processing
	#[error("io error while encoding block")]
	IoError(#[from] std::io::Error),

	/// We could not encode a picture inside a vorbis comment
	#[error("could not encode picture in vorbis comment")]
	VorbisPictureEncodeError,
}

impl From<VorbisCommentEncodeError> for FlacEncodeError {
	fn from(value: VorbisCommentEncodeError) -> Self {
		match value {
			VorbisCommentEncodeError::IoError(e) => e.into(),
			VorbisCommentEncodeError::PictureEncodeError => Self::VorbisPictureEncodeError,
		}
	}
}
