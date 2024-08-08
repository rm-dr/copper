use crate::{flac::errors::FlacError, FileBlockDecode};

// TODO: parse

/// A seektable block in a flac file
pub struct FlacSeektableBlock {
	/// The seek table
	pub data: Vec<u8>,
}

impl FileBlockDecode for FlacSeektableBlock {
	type DecodeErrorType = FlacError;

	fn decode(data: &[u8]) -> Result<Self, Self::DecodeErrorType> {
		Ok(Self { data: data.into() })
	}
}
