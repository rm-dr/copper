use crate::{flac::errors::FlacError, FileBlockDecode};
use std::io::{Cursor, Read};

/// An application block in a flac file
pub struct FlacApplicationBlock {
	/// Registered application ID
	pub application_id: u32,

	/// The application data
	pub data: Vec<u8>,
}

impl FileBlockDecode for FlacApplicationBlock {
	type DecodeErrorType = FlacError;

	fn decode(data: &[u8]) -> Result<Self, Self::DecodeErrorType> {
		let mut d = Cursor::new(data);

		let mut block = [0u8; 4];
		d.read_exact(&mut block)
			.map_err(|_| FlacError::MalformedBlock)?;
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
