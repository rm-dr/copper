//! Parse FLAC metadata.

use std::io::{Read, Seek, SeekFrom};

use self::{errors::FlacError, metablocktype::FlacMetablockType, picture::FlacPicture};
use crate::common::vorbiscomment::VorbisComment;

pub mod errors;
pub mod metablocktype;
pub mod metastrip;
pub mod picture;
pub mod streaminfo;

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

/// Try to extract flac pictures from the given reader.
/// `read` should provide a complete FLAC file.
pub fn flac_read_pictures<'a, R>(mut read: R) -> Result<Option<FlacPicture<'a>>, FlacError>
where
	R: Read + Seek + 'a,
{
	let mut block = [0u8; 4];
	read.read_exact(&mut block)?;
	if block != [0x66, 0x4C, 0x61, 0x43] {
		return Err(FlacError::BadMagicBytes);
	};

	// TODO: what if we have multiple pictures?
	// How about pictures in vorbis blocks?
	loop {
		let (block_type, length, is_last) = FlacMetablockType::parse_header(&mut read)?;

		match block_type {
			FlacMetablockType::Picture => {
				return Ok(Some(FlacPicture::decode(read.take(length.into())).unwrap()));
			}
			_ => {
				if is_last {
					break;
				} else {
					read.seek(SeekFrom::Current(length.into()))?;
					continue;
				}
			}
		};
	}

	return Ok(None);
}
