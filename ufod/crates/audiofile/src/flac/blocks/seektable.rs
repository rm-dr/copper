use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A seektable block in a flac file
pub struct FlacSeektableBlock {
	/// The seek table
	pub data: Vec<u8>,
}

impl FlacMetablockDecode for FlacSeektableBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		Ok(Self { data: data.into() })
	}
}

impl FlacMetablockEncode for FlacSeektableBlock {
	fn encode(
		&self,
		is_last: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		let header = FlacMetablockHeader {
			block_type: FlacMetablockType::Seektable,
			length: self.data.len().try_into().unwrap(),
			is_last,
		};

		header.encode(target)?;
		target.write_all(&self.data)?;
		return Ok(());
	}
}
