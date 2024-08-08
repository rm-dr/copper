//! FLAC metablock types. See spec.

use std::io::Read;

use super::errors::FlacError;

/// See FLAC spec
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
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
	pub(crate) fn make_header(&self, is_last: bool, length: u32) -> [u8; 4] {
		let mut block_type = match self {
			FlacMetablockType::Streaminfo => 0,
			FlacMetablockType::Padding => 1,
			FlacMetablockType::Application => 2,
			FlacMetablockType::Seektable => 3,
			FlacMetablockType::VorbisComment => 4,
			FlacMetablockType::Cuesheet => 5,
			FlacMetablockType::Picture => 6,
		};

		if is_last {
			block_type |= 0b1000_0000;
		};

		let x = length.to_be_bytes();
		return [block_type, x[1], x[2], x[3]];
	}

	/// Read and parse a metablock header from the given reader.
	/// Returns (block_type, block_data_length, is_last)
	pub(crate) fn parse_header<R>(mut read: R) -> Result<(Self, u32, bool), FlacError>
	where
		R: Read,
	{
		let mut block = [0u8; 4];
		read.read_exact(&mut block)?;

		// Last-metadata-block flag:
		// '1' if this block is the last metadata block before the audio blocks,
		// '0' otherwise.
		let is_last = block[0] & 0b10000000 == 0b10000000;
		let block_type = match block[0] & 0b01111111 {
			0 => FlacMetablockType::Streaminfo,
			1 => FlacMetablockType::Padding,
			2 => FlacMetablockType::Application,
			3 => FlacMetablockType::Seektable,
			4 => FlacMetablockType::VorbisComment,
			5 => FlacMetablockType::Cuesheet,
			6 => FlacMetablockType::Picture,
			x => return Err(FlacError::BadMetablockType(x)),
		};
		let length = u32::from_be_bytes([0, block[1], block[2], block[3]]);

		return Ok((block_type, length, is_last));
	}
}
