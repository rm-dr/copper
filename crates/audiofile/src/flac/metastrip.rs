//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::io::{Read, Seek, SeekFrom};

use super::{errors::FlacError, metablocktype::FlacMetablockType};

// TODO: tests
// TODO: select blocks to keep
// TODO: implement Seek

/// Given a reader to flac data, write another flac file
/// with all non-essential metadata flags stripped.
///
/// Note that this isn't designed to write data to the filesystem:
/// This does NOT add padding frames---thus, editing the tags
/// of the resulting file requires us to re-write the whole file again.
///
/// Rather, this prepares a flac file for storage in another format,
/// where tags are stored seperately (a database, for example).
pub struct FlacMetaStrip<R>
where
	R: Read + Seek,
{
	// The old file
	read: R,

	// All blocks we want to keep.
	// Format: (type, position_in_old_file, len_in_bytes)
	// These must be in order.
	blocks: Vec<(FlacMetablockType, u64, u32)>,

	// Where we are in the new stream
	position: u64,

	// The number of bytes in the old file's metadata
	// (including magic bytes)
	old_meta_len: u64,

	// The number of bytes in the new file's metadata
	// (including magic bytes)
	new_meta_len: u64,

	// The number of bytes in the new file
	// (including metadata)
	new_total_len: u64,
}

impl<R: Read + Seek> FlacMetaStrip<R> {
	/// Create an object that strips tags from the given reader.
	/// `read` should be a complete, valid FLAC file.
	pub fn new(mut read: R) -> Result<Self, FlacError> {
		let mut block = [0u8; 4];
		read.read_exact(&mut block)?;
		if block != [0x66, 0x4C, 0x61, 0x43] {
			return Err(FlacError::BadMagicBytes);
		};

		let mut blocks = Vec::new();
		let mut new_meta_len = 4u64; // Initial 4 bytes for "fLaC" header
		let mut old_meta_len = 4u64;
		loop {
			let (block_type, length, is_last) = FlacMetablockType::parse_header(&mut read)?;

			let keep_block = match block_type {
				FlacMetablockType::Streaminfo => true,
				FlacMetablockType::Padding => false,
				FlacMetablockType::Application => false,
				FlacMetablockType::Seektable => true,
				FlacMetablockType::VorbisComment => false,
				FlacMetablockType::Cuesheet => true,
				FlacMetablockType::Picture => false,
			};

			old_meta_len += 4 + u64::from(length);
			if keep_block {
				blocks.push((block_type, read.stream_position()?, length));
				new_meta_len += 4 + u64::from(length);
			}

			if is_last {
				break;
			} else {
				read.seek(SeekFrom::Current(length.into()))?;
				continue;
			}
		}

		let x = read.seek(SeekFrom::End(0))?;

		Ok(Self {
			read,
			blocks,
			position: 0,
			new_meta_len,
			old_meta_len,
			new_total_len: x - old_meta_len + new_meta_len,
		})
	}
}

// TODO: test this implementation
impl<R: Read + Seek> Seek for FlacMetaStrip<R> {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		match pos {
			SeekFrom::Current(x) => {
				let p: u64 = x.abs().try_into().unwrap();
				if x >= 0 {
					self.position += p;
				} else {
					if p < self.position {
						return Err(std::io::Error::new(
							std::io::ErrorKind::InvalidInput,
							"invalid seek to a negative or overflowing position",
						));
					}
					self.position -= p;
				}
			}
			SeekFrom::Start(x) => self.position = x,
			SeekFrom::End(x) => {
				let p: u64 = x.abs().try_into().unwrap();
				if x >= 0 {
					self.position = self.new_total_len + p;
				} else {
					if p < self.new_total_len {
						return Err(std::io::Error::new(
							std::io::ErrorKind::InvalidInput,
							"invalid seek to a negative or overflowing position",
						));
					}
					self.position = self.new_total_len - p;
				}
			}
		}

		Ok(self.position)
	}
}

impl<R: Read + Seek> Read for FlacMetaStrip<R> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let mut bytes_written = 0;

		// Write magic bytes
		if self.position <= 3 {
			let magic_bytes = [0x66, 0x4C, 0x61, 0x43];

			let space_left = buf.len() - bytes_written;
			let n_to_write = space_left.min(4);
			let start_at = usize::try_from(self.position).unwrap();
			for i in start_at..n_to_write {
				buf[bytes_written] = magic_bytes[i];
				bytes_written += 1;
				self.position += 1;
			}

			if bytes_written == buf.len() {
				return Ok(bytes_written);
			}
		}
		assert!(bytes_written < buf.len());

		// Write the metablocks we're keeping
		while self.position < self.new_meta_len {
			// Find which block we're in
			let mut current_block_idx = 0usize;
			let mut current_block_start = 4u64;
			for (_, _, l) in &self.blocks {
				let lx = u64::from(*l);
				if self.position < (current_block_start + lx + 4) {
					break;
				} else {
					current_block_start += lx + 4;
					current_block_idx += 1;
				}
			}

			let byte_in_block = self.position - current_block_start;

			// Write metablock header
			let byte_in_block = if byte_in_block <= 3 {
				let header = self.blocks[current_block_idx].0.make_header(
					current_block_idx == self.blocks.len() - 1,
					self.blocks[current_block_idx].2,
				);

				let space_left = buf.len() - bytes_written;
				let n_to_write = space_left.min(4);
				let start_at = usize::try_from(byte_in_block).unwrap();
				for i in start_at..n_to_write {
					buf[bytes_written] = header[i];
					bytes_written += 1;
					self.position += 1;
				}

				if bytes_written == buf.len() {
					return Ok(bytes_written);
				}
				0
			} else {
				byte_in_block - 4
			};

			// Write metablock data
			self.read.seek(SeekFrom::Start(
				self.blocks[current_block_idx].1 + byte_in_block,
			))?;

			let mut c = self
				.read
				.by_ref()
				.take(u64::from(self.blocks[current_block_idx].2) - byte_in_block);

			let l = c.read(&mut buf[bytes_written..])?;

			self.position += u64::try_from(l).unwrap();
			bytes_written += l;
			if bytes_written == buf.len() {
				return Ok(bytes_written);
			}
		}
		assert!(bytes_written < buf.len());

		// Write frames
		if self.position >= self.new_meta_len {
			let pos_in_data = self.position - self.new_meta_len;
			let pos_in_old = self.old_meta_len + pos_in_data;
			self.read.seek(SeekFrom::Start(pos_in_old))?;
			let l = self.read.read(&mut buf[bytes_written..])?;
			self.position += u64::try_from(l).unwrap();
			bytes_written += l;
			if bytes_written == buf.len() {
				return Ok(bytes_written);
			}
		}
		return Ok(bytes_written);
	}
}
