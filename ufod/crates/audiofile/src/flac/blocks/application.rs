use crate::flac::errors::{FlacDecodeError, FlacEncodeError};
use std::io::{Cursor, Read};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// An application block in a flac file
pub struct FlacApplicationBlock {
	/// Registered application ID
	pub application_id: u32,

	/// The application data
	pub data: Vec<u8>,
}

impl FlacMetablockDecode for FlacApplicationBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		let mut d = Cursor::new(data);

		let mut block = [0u8; 4];
		d.read_exact(&mut block)
			.map_err(|_| FlacDecodeError::MalformedBlock)?;
		let application_id = u32::from_be_bytes(block);

		let data = {
			let mut data = Vec::with_capacity(data.len());
			d.read_to_end(&mut data)?;
			data
		};

		Ok(Self {
			application_id,
			data,
		})
	}
}

impl FlacMetablockEncode for FlacApplicationBlock {
	fn encode(
		&self,
		is_last: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		let header = FlacMetablockHeader {
			block_type: FlacMetablockType::Application,
			length: (self.data.len() + 4).try_into().unwrap(),
			is_last,
		};

		header.encode(target)?;
		target.write_all(&self.application_id.to_be_bytes())?;
		target.write_all(&self.data)?;
		return Ok(());
	}
}
