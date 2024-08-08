use std::fmt::Debug;

use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A padding block in a FLAC file.
#[derive(Debug)]
pub struct FlacPaddingBlock {
	/// The length of this padding, in bytes.
	pub size: usize,
}

impl FlacMetablockDecode for FlacPaddingBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		if data.iter().any(|x| *x != 0u8) {
			return Err(FlacDecodeError::MalformedBlock);
		}

		Ok(Self { size: data.len() })
	}
}

impl FlacMetablockEncode for FlacPaddingBlock {
	fn encode(
		&self,
		is_last: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		let header = FlacMetablockHeader {
			block_type: FlacMetablockType::Padding,
			length: self.size.try_into().unwrap(),
			is_last,
		};

		header.encode(target)?;

		// TODO: Don't allocate a ton of zeros.
		let zeros: Vec<u8> = std::iter::repeat(0u8).take(self.size).collect();
		target.write_all(&zeros)?;
		return Ok(());
	}
}
