//! FLAC metablock headers. See spec.
use crate::{flac::errors::FlacError, FileBlockDecode, FileBlockEncode};

// TODO: enfoce length limits

/// A type of flac metadata block
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FlacMetablockType {
	Streaminfo,
	Padding,
	Application,
	Seektable,
	VorbisComment,
	Cuesheet,
	Picture,
}

impl FlacMetablockType {
	/// Read and parse a metablock header from the given reader.
	/// Returns (block_type, block_data_length, is_last)
	pub(crate) fn from_id(id: u8) -> Result<Self, FlacError> {
		return Ok(match id & 0b01111111 {
			0 => FlacMetablockType::Streaminfo,
			1 => FlacMetablockType::Padding,
			2 => FlacMetablockType::Application,
			3 => FlacMetablockType::Seektable,
			4 => FlacMetablockType::VorbisComment,
			5 => FlacMetablockType::Cuesheet,
			6 => FlacMetablockType::Picture,
			x => return Err(FlacError::BadMetablockType(x)),
		});
	}
}

/// The header of a flac metadata block
pub struct FlacMetablockHeader {
	/// The type of block this is
	pub block_type: FlacMetablockType,

	/// The length of this block, in bytes
	/// (not including this header)
	pub length: u32,

	/// If true, this is the last metadata block
	pub is_last: bool,
}

impl FileBlockDecode for FlacMetablockHeader {
	type DecodeErrorType = FlacError;

	fn decode(header: &[u8]) -> Result<Self, Self::DecodeErrorType> {
		if header.len() != 4 {
			return Err(FlacError::MalformedBlock);
		}

		return Ok(Self {
			block_type: FlacMetablockType::from_id(header[0])?,
			length: u32::from_be_bytes([0, header[1], header[2], header[3]]),
			is_last: header[0] & 0b10000000 == 0b10000000,
		});
	}
}

impl FileBlockEncode for FlacMetablockHeader {
	type EncodeErrorType = std::convert::Infallible;

	fn encode(&self) -> Result<Vec<u8>, Self::EncodeErrorType> {
		let mut block_type = match self.block_type {
			FlacMetablockType::Streaminfo => 0,
			FlacMetablockType::Padding => 1,
			FlacMetablockType::Application => 2,
			FlacMetablockType::Seektable => 3,
			FlacMetablockType::VorbisComment => 4,
			FlacMetablockType::Cuesheet => 5,
			FlacMetablockType::Picture => 6,
		};

		if self.is_last {
			block_type |= 0b1000_0000;
		};

		let x = self.length.to_be_bytes();
		return Ok(vec![block_type, x[1], x[2], x[3]]);
	}
}
