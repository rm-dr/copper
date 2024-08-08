//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::{
	collections::VecDeque,
	io::{Cursor, Read, Write},
};

use super::metablocktype::FlacMetablockType;

// TODO: tests
// TODO: select blocks to keep
// TODO: implement Seek

#[derive(Debug, Clone, Copy)]
enum FlacMetaStripBlockType {
	MagicBits,
	BlockHeader,
	MetaBlock {
		header: [u8; 4],
		keep_this_block: bool,
	},
	AudioData,
}

impl FlacMetaStripBlockType {
	fn is_audiodata(&self) -> bool {
		matches!(self, Self::AudioData)
	}
}

/// A buffered flac metadata stripper.
/// `Write` flac data into this struct,
/// `Read` the same flac data but with metadata removed.
pub struct FlacMetaStrip {
	// The block we're currently reading
	current_block: Vec<u8>,

	// The total length of the block we're currently reading.
	current_block_total_length: usize,

	// The number of bytes we're currently written to `current_block_type`.
	// This is usually equal to `current_block_type.len()`, except for when
	// we fake-read blocks we ignore.
	current_block_length: usize,

	// The type of the block we're currently reading
	current_block_type: FlacMetaStripBlockType,

	// If `true`, we've read all metadata blocks
	done_with_meta: bool,

	// The last block we kept.
	// Used to mark the "is_last" metadata bit.
	last_kept_block: Option<([u8; 4], Vec<u8>)>,

	// Flac data with removed blocks goes here.
	output_buffer: VecDeque<u8>,
}

impl FlacMetaStrip {
	/// Make a new [`FlacMetaStrip`].
	pub fn new() -> Self {
		Self {
			current_block: Vec::new(),
			current_block_total_length: 4,
			current_block_type: FlacMetaStripBlockType::MagicBits,
			done_with_meta: false,
			last_kept_block: None,
			current_block_length: 0,
			output_buffer: VecDeque::new(),
		}
	}
}

impl Write for FlacMetaStrip {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let mut buf = Cursor::new(buf);
		let mut written: usize = 0;

		loop {
			// If we've read all metadata and aren't currently reading a block,
			// write directly to output.
			if self.current_block_type.is_audiodata() {
				return Ok(
					usize::try_from(std::io::copy(&mut buf, &mut self.output_buffer)?).unwrap()
						+ written,
				);
			}

			let current_block_left = self.current_block_total_length - self.current_block_length;

			if current_block_left == 0 {
				// If we filled this block, clean up and start the next one.

				match self.current_block_type {
					FlacMetaStripBlockType::MetaBlock {
						header,
						keep_this_block,
					} => {
						// If we're keeping this block, we know that the previously
						// kept block wasn't last. Write it to output and replace it
						// with this block.
						if keep_this_block {
							assert!(self.current_block_length == self.current_block.len());
							if let Some((header, block)) = self.last_kept_block.take() {
								self.output_buffer.extend(header);
								self.output_buffer.extend(block);
							}
							self.last_kept_block = Some((
								header,
								std::mem::replace(&mut self.current_block, Vec::new()),
							));
						}

						if self.done_with_meta {
							// We just read the last metadata block.
							// Append last_kept_block and prepare to read audio data
							if let Some((header, block)) = self.last_kept_block.take() {
								let x = FlacMetablockType::parse_header(&header[..]);
								let (block_type, length, _) = x.unwrap();
								self.output_buffer
									.extend(block_type.make_header(true, length));
								self.output_buffer.extend(block);
							}
							self.current_block_total_length = 0;
							self.current_block_type = FlacMetaStripBlockType::AudioData;
						} else {
							// We have another metadata block to read,
							// prepare to read the header.
							self.current_block_total_length = 4;
							self.current_block_type = FlacMetaStripBlockType::BlockHeader;
						}
					}

					FlacMetaStripBlockType::MagicBits => {
						assert!(self.current_block.len() == 4);
						assert!(self.current_block_length == 4);
						if self.current_block != [0x66, 0x4C, 0x61, 0x43] {
							panic!() //TODO: error
							 //return Err(FlacError::BadMagicBytes);
						};
						self.output_buffer.extend(&self.current_block);
						self.current_block_total_length = 4;
						self.current_block_type = FlacMetaStripBlockType::BlockHeader;
					}

					FlacMetaStripBlockType::BlockHeader => {
						assert!(self.current_block.len() == 4);
						assert!(self.current_block_length == 4);
						let x = FlacMetablockType::parse_header(&self.current_block[..]);
						let (block_type, length, is_last) = x.unwrap(); // TODO: handle errors
						self.done_with_meta = is_last;
						self.current_block_total_length = length.try_into().unwrap();

						let keep_next_block = match block_type {
							FlacMetablockType::Streaminfo => true,
							FlacMetablockType::Padding => false,
							FlacMetablockType::Application => false,
							FlacMetablockType::Seektable => true,
							FlacMetablockType::VorbisComment => false,
							FlacMetablockType::Cuesheet => true,
							FlacMetablockType::Picture => false,
						};

						self.current_block_type = FlacMetaStripBlockType::MetaBlock {
							header: self.current_block[..].try_into().unwrap(),
							keep_this_block: keep_next_block,
						};
					}

					FlacMetaStripBlockType::AudioData => unreachable!(),
				}

				self.current_block.clear();
				self.current_block_length = 0;
			} else {
				// Minor optimization:
				// Don't even read blocks we're skipping.
				let really_read = match self.current_block_type {
					FlacMetaStripBlockType::MetaBlock {
						keep_this_block, ..
					} => keep_this_block,
					_ => true,
				};

				// Otherwise, keep reading.
				let read = usize::try_from(if really_read {
					std::io::copy(
						&mut buf.by_ref().take(current_block_left.try_into().unwrap()),
						&mut self.current_block,
					)
				} else {
					std::io::copy(
						&mut buf.by_ref().take(current_block_left.try_into().unwrap()),
						&mut std::io::empty(),
					)
				}?)
				.unwrap();
				self.current_block_length += read;

				if read == 0 {
					return Ok(written);
				} else {
					written += read;
				}
			}
		}
	}

	fn flush(&mut self) -> std::io::Result<()> {
		return Ok(());
	}
}

impl Read for FlacMetaStrip {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let n_to_read = buf.len().min(self.output_buffer.len());

		let x = Read::by_ref(&mut self.output_buffer)
			.take(n_to_read.try_into().unwrap())
			.read(buf)?;
		//self.output_buffer.drain(0..x);
		return Ok(x);
	}
}
