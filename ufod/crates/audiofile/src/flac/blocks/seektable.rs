use std::fmt::Debug;

use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A seektable block in a flac file
pub struct FlacSeektableBlock {
	/// The seek table
	pub data: Vec<u8>,
}

impl Debug for FlacSeektableBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FlacSeektableBlock")
			.field("data_len", &self.data.len())
			.finish()
	}
}

impl FlacMetablockDecode for FlacSeektableBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		Ok(Self { data: data.into() })
	}
}

impl FlacMetablockEncode for FlacSeektableBlock {
	fn get_len(&self) -> u32 {
		self.data.len().try_into().unwrap()
	}

	fn encode(
		&self,
		is_last: bool,
		with_header: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		if with_header {
			let header = FlacMetablockHeader {
				block_type: FlacMetablockType::Seektable,
				length: self.get_len(),
				is_last,
			};
			header.encode(target)?;
		}

		target.write_all(&self.data)?;
		return Ok(());
	}
}
