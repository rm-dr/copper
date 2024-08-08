use std::fmt::Debug;

use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A cuesheet meta in a flac file
pub struct FlacCuesheetBlock {
	/// The seek table
	pub data: Vec<u8>,
}

impl Debug for FlacCuesheetBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FlacAudioFrame")
			.field("data_len", &self.data.len())
			.finish()
	}
}

impl FlacMetablockDecode for FlacCuesheetBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		Ok(Self { data: data.into() })
	}
}

impl FlacMetablockEncode for FlacCuesheetBlock {
	fn encode(
		&self,
		is_last: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		let header = FlacMetablockHeader {
			block_type: FlacMetablockType::Cuesheet,
			length: self.data.len().try_into().unwrap(),
			is_last,
		};

		header.encode(target)?;
		target.write_all(&self.data)?;
		return Ok(());
	}
}
