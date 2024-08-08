use crate::{flac::errors::FlacError, FileBlockDecode};

// TODO: parse

/// A cuesheet meta in a flac file
pub struct FlacCuesheetBlock {
	/// The seek table
	pub data: Vec<u8>,
}

impl FileBlockDecode for FlacCuesheetBlock {
	type DecodeErrorType = FlacError;

	fn decode(data: &[u8]) -> Result<Self, Self::DecodeErrorType> {
		Ok(Self { data: data.into() })
	}
}
