use std::{fmt::Debug, io::Read};

use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A padding block in a FLAC file.
#[derive(Debug)]
pub struct FlacPaddingBlock {
	/// The length of this padding, in bytes.
	pub size: u32,
}

impl FlacMetablockDecode for FlacPaddingBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		if data.iter().any(|x| *x != 0u8) {
			return Err(FlacDecodeError::MalformedBlock);
		}

		Ok(Self {
			size: data.len().try_into().unwrap(),
		})
	}
}

impl FlacMetablockEncode for FlacPaddingBlock {
	fn get_len(&self) -> u32 {
		self.size
	}

	fn encode(
		&self,
		is_last: bool,
		with_header: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		if with_header {
			let header = FlacMetablockHeader {
				block_type: FlacMetablockType::Padding,
				length: self.get_len(),
				is_last,
			};
			header.encode(target)?;
		}

		std::io::copy(&mut std::io::repeat(0u8).take(self.size.into()), target)?;

		return Ok(());
	}
}
