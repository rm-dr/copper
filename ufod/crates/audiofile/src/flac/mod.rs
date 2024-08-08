//! Parse FLAC metadata.

use std::io::{Read, Seek, SeekFrom};

use self::errors::FlacError;
use crate::{common::vorbiscomment::VorbisComment, FileBlockDecode};

pub mod blocks;
pub mod errors;
pub mod metastrip;
pub mod picture;

use blocks::{FlacMetablockHeader, FlacMetablockType};

/// Try to extract a vorbis comment block from the given reader.
/// `read` should provide a complete FLAC file.
pub fn flac_read_tags<R>(mut read: R) -> Result<Option<VorbisComment>, FlacError>
where
	R: Read + Seek,
{
	let mut block = [0u8; 4];
	read.read_exact(&mut block)?;
	if block != [0x66, 0x4C, 0x61, 0x43] {
		return Err(FlacError::BadMagicBytes);
	};

	// TODO: what if we have multiple vorbis blocks?
	let mut header = [0u8; 4];

	loop {
		read.read_exact(&mut header)?;
		let h = FlacMetablockHeader::decode(&header)?;

		match h.block_type {
			FlacMetablockType::VorbisComment => {
				return Ok(Some(VorbisComment::decode(read.take(h.length.into()))?));
			}
			_ => {
				read.seek(SeekFrom::Current(h.length.into()))?;
			}
		};

		if h.is_last {
			break;
		}
	}

	return Ok(None);
}
