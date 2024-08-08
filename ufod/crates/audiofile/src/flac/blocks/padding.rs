use crate::{flac::errors::FlacError, FileBlockDecode};

/// A padding block in a FLAC file.
pub struct FlacPaddingBlock {
	/// The length of this padding, in bytes.
	pub size: usize,
}

impl FileBlockDecode for FlacPaddingBlock {
	type DecodeErrorType = FlacError;

	fn decode(data: &[u8]) -> Result<Self, Self::DecodeErrorType> {
		if data.iter().any(|x| *x != 0u8) {
			return Err(FlacError::MalformedBlock);
		}

		Ok(Self { size: data.len() })
	}
}
