use std::io::{Read, Seek, SeekFrom};

use self::{errors::FlacError, metablocktype::FlacMetablockType};
use crate::common::vorbiscomment::VorbisComment;

pub mod errors;
pub mod metablocktype;
pub mod metastripper;
pub mod picture;
pub mod streaminfo;

pub fn flac_read_tags<R>(mut read: R) -> Result<Option<VorbisComment>, FlacError>
where
	R: Read + Seek,
{
	let mut block = [0u8; 4];
	read.read_exact(&mut block)?;
	if block != [0x66, 0x4C, 0x61, 0x43] {
		return Err(FlacError::BadMagicBytes);
	};

	loop {
		let (block_type, length, is_last) = FlacMetablockType::parse_header(&mut read)?;

		match block_type {
			FlacMetablockType::VorbisComment => {
				return Ok(Some(VorbisComment::decode(read.take(length.into()))?));
			}
			_ => {
				if is_last {
					break;
				} else {
					// Skip without seek:
					// io::copy(&mut file.by_ref().take(27), &mut io::sink());
					read.seek(SeekFrom::Current(length.into()))?;
					continue;
				}
			}
		};
	}

	return Ok(None);
}
