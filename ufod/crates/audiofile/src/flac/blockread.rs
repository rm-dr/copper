//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::{
	collections::VecDeque,
	io::{Cursor, Read, Seek},
};

use super::{
	blocks::{FlacMetablockHeader, FlacMetablockType},
	errors::FlacError,
};
use crate::{
	common::vorbiscomment::VorbisComment,
	flac::blocks::{
		FlacApplicationBlock, FlacCuesheetBlock, FlacPaddingBlock, FlacPictureBlock,
		FlacSeektableBlock, FlacStreaminfoBlock,
	},
	FileBlockDecode,
};

/// Select which blocks we want to keep.
/// All values are `false` by default.
#[derive(Debug, Default, Clone, Copy)]
pub struct FlacBlockSelector {
	/// Select `FlacMetablockType::Streaminfo` blocks.
	pub pick_streaminfo: bool,

	/// Select `FlacMetablockType::Padding` blocks.
	pub pick_padding: bool,

	/// Select `FlacMetablockType::Application` blocks.
	pub pick_application: bool,

	/// Select `FlacMetablockType::SeekTable` blocks.
	pub pick_seektable: bool,

	/// Select `FlacMetablockType::VorbisComment` blocks.
	pub pick_vorbiscomment: bool,

	/// Select `FlacMetablockType::CueSheet` blocks.
	pub pick_cuesheet: bool,

	/// Select `FlacMetablockType::Picture` blocks.
	pub pick_picture: bool,

	/// Select audio frames.
	pub pick_audio: bool,
}

impl FlacBlockSelector {
	/// Make a new [`FlacBlockSelector`]
	pub fn new() -> Self {
		Self::default()
	}

	fn should_pick_meta(&self, block_type: FlacMetablockType) -> bool {
		match block_type {
			FlacMetablockType::Streaminfo => self.pick_streaminfo,
			FlacMetablockType::Padding => self.pick_padding,
			FlacMetablockType::Application => self.pick_application,
			FlacMetablockType::Seektable => self.pick_seektable,
			FlacMetablockType::VorbisComment => self.pick_vorbiscomment,
			FlacMetablockType::Cuesheet => self.pick_cuesheet,
			FlacMetablockType::Picture => self.pick_picture,
		}
	}
}

enum FlacBlockType {
	MagicBits {
		data: [u8; 4],
		left_to_read: usize,
	},
	MetablockHeader {
		is_first: bool,
		data: [u8; 4],
		left_to_read: usize,
	},
	MetaBlock {
		header: FlacMetablockHeader,
		data: Vec<u8>,
	},
	AudioData {
		data: Vec<u8>,
	},
}

#[allow(missing_docs)]
pub enum FlacBlock {
	Streaminfo(FlacStreaminfoBlock),
	Picture(FlacPictureBlock),
	Padding(FlacPaddingBlock),
	Application(FlacApplicationBlock),
	SeekTable(FlacSeektableBlock),
	VorbisComment(VorbisComment),
	CueSheet(FlacCuesheetBlock),
	AudioFrame(Vec<u8>),
}

/// A buffered flac block reader.
/// Use `push_data` to add flac data into this struct,
/// use `pop_block` to read flac blocks.
///
/// This struct does not validate the content of the blocks it produces;
/// it only validates their structure (e.g, is length correct?).
pub struct FlacBlockReader {
	// Which blocks should we return?
	selector: FlacBlockSelector,

	// The block we're currently reading.
	// If this is `None`, we've called `finish()`.
	current_block: Option<FlacBlockType>,

	// Blocks we pick go here
	output_blocks: VecDeque<FlacBlock>,
}

impl FlacBlockReader {
	/// Pop the next block we've read, if any.
	pub fn pop_block(&mut self) -> Option<FlacBlock> {
		self.output_blocks.pop_front()
	}

	/// Make a new [`FlacBlockReader`].
	pub fn new(selector: FlacBlockSelector) -> Self {
		Self {
			selector,
			current_block: Some(FlacBlockType::MagicBits {
				data: [0; 4],
				left_to_read: 4,
			}),

			output_blocks: VecDeque::new(),
		}
	}

	/// Pass the given data through this block extractor.
	/// Output data is stored in an internal buffer, and should be accessed
	/// through `Read`.
	pub fn push_data(&mut self, buf: &[u8]) -> Result<(), FlacError> {
		let mut buf = Cursor::new(buf);
		let mut last_read_size = 1;

		if self.current_block.is_none() {
			panic!("Tried to push data to a finished reader")
		}

		'outer: while last_read_size != 0 {
			match self.current_block.as_mut().unwrap() {
				FlacBlockType::MagicBits { data, left_to_read } => {
					last_read_size = buf.read(&mut data[4 - *left_to_read..4]).unwrap();
					*left_to_read -= last_read_size;

					if *left_to_read == 0 {
						if *data != [0x66, 0x4C, 0x61, 0x43] {
							return Err(FlacError::BadMagicBytes);
						};

						self.current_block = Some(FlacBlockType::MetablockHeader {
							is_first: true,
							data: [0; 4],
							left_to_read: 4,
						})
					}
				}

				FlacBlockType::MetablockHeader {
					is_first,
					data,
					left_to_read,
				} => {
					last_read_size = buf.read(&mut data[4 - *left_to_read..4]).unwrap();
					*left_to_read -= last_read_size;

					if *left_to_read == 0 {
						let header = FlacMetablockHeader::decode(data)?;
						if *is_first && !matches!(header.block_type, FlacMetablockType::Streaminfo)
						{
							return Err(FlacError::BadFirstBlock);
						}

						self.current_block = Some(FlacBlockType::MetaBlock {
							header,
							data: Vec::new(),
						})
					}
				}

				FlacBlockType::MetaBlock { header, data } => {
					last_read_size = buf
						.by_ref()
						.take(u64::from(header.length) - u64::try_from(data.len()).unwrap())
						.read_to_end(data)
						.unwrap();

					if data.len() == header.length.try_into().unwrap() {
						// If we picked this block type, add it to the queue
						if self.selector.should_pick_meta(header.block_type) {
							let b = match header.block_type {
								FlacMetablockType::Streaminfo => {
									FlacBlock::Streaminfo(FlacStreaminfoBlock::decode(&data)?)
								}
								FlacMetablockType::Application => {
									FlacBlock::Application(FlacApplicationBlock::decode(&data)?)
								}
								FlacMetablockType::Cuesheet => {
									FlacBlock::CueSheet(FlacCuesheetBlock::decode(&data)?)
								}
								FlacMetablockType::Padding => {
									FlacBlock::Padding(FlacPaddingBlock::decode(&data)?)
								}
								FlacMetablockType::Picture => {
									FlacBlock::Picture(FlacPictureBlock::decode(&data)?)
								}
								FlacMetablockType::Seektable => {
									FlacBlock::SeekTable(FlacSeektableBlock::decode(&data)?)
								}
								FlacMetablockType::VorbisComment => {
									FlacBlock::VorbisComment(VorbisComment::decode(&data)?)
								}
							};

							self.output_blocks.push_back(b);
						}

						// Start next block
						if header.is_last {
							self.current_block = Some(FlacBlockType::AudioData { data: Vec::new() })
						} else {
							self.current_block = Some(FlacBlockType::MetablockHeader {
								is_first: false,
								data: [0; 4],
								left_to_read: 4,
							})
						}
					}
				}

				FlacBlockType::AudioData { data } => {
					// Limit the number of bytes we read at once, so we don't re-clone
					// large amounts of data if `buf` contains multiple sync sequences.
					// 5kb is a pretty reasonable frame size.
					last_read_size = buf.by_ref().take(5_000).read_to_end(data).unwrap();
					if last_read_size == 0 {
						continue 'outer;
					}

					// We can't run checks if we don't have enough data.
					if data.len() <= 2 {
						continue;
					}

					// Check frame sync header
					// (`if` makes sure we only do this once)
					if data.len() - last_read_size <= 2 {
						if !(data[0] == 0b1111_1111 && data[1] & 0b1111_1100 == 0b1111_1000) {
							return Err(FlacError::BadSyncBytes);
						}
					}

					if data.len() > 2 {
						// Look for a frame sync header in the data we read
						let first_byte = if last_read_size + 2 > data.len() {
							1
						} else {
							data.len() - (last_read_size + 2)
						};

						for i in first_byte..(data.len() - 2) {
							if data[i] == 0b1111_1111 && data[i + 1] & 0b1111_1100 == 0b1111_1000 {
								// We found another frame sync header. Split at this index.
								if self.selector.pick_audio {
									self.output_blocks
										.push_back(FlacBlock::AudioFrame(Vec::from(&data[0..i])));
								}

								// Backtrack to the first bit of this new sync sequence
								buf.seek(std::io::SeekFrom::Current(
									-i64::try_from(data.len() - i).unwrap(),
								))?;

								self.current_block =
									Some(FlacBlockType::AudioData { data: Vec::new() });
								continue 'outer;
							}
						}
					}
				}
			}
		}

		return Ok(());
	}

	/// Finish reading data.
	/// This tells the reader that it has received the entire stream.
	pub fn finish(&mut self) -> Result<(), FlacError> {
		match self.current_block.take() {
			None => {
				panic!("Called `finish()` on a finished reader")
			}

			Some(FlacBlockType::AudioData { data }) => {
				// We can't run checks if we don't have enough data.
				if data.len() <= 2 {
					return Err(FlacError::MalformedBlock);
				}

				if !(data[0] == 0b1111_1111 && data[1] & 0b1111_1100 == 0b1111_1000) {
					return Err(FlacError::BadSyncBytes);
				}

				if self.selector.pick_audio {
					self.output_blocks.push_back(FlacBlock::AudioFrame(data));
				}

				self.current_block = None;
				return Ok(());
			}

			// All other blocks have a known length, and
			// are finished automatically.
			_ => return Err(FlacError::MalformedBlock),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::path::PathBuf;

	#[test]
	fn strip_test_whole() -> Result<(), FlacError> {
		let selector = FlacBlockSelector {
			pick_streaminfo: true,
			pick_padding: true,
			pick_application: true,
			pick_seektable: true,
			pick_vorbiscomment: true,
			pick_cuesheet: true,
			pick_picture: true,
			pick_audio: true,
		};

		let test_file_path = &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_custom/01 - many images.flac");

		let file_data = std::fs::read(test_file_path).unwrap();
		let mut x = FlacBlockReader::new(selector);

		x.push_data(&file_data)?;
		while let Some(b) = x.pop_block() {
			println!(
				"{:?}",
				match b {
					FlacBlock::Application(_) => "a",
					FlacBlock::Streaminfo(_) => "i",
					FlacBlock::AudioFrame(_) => "f",
					FlacBlock::CueSheet(_) => "c",
					FlacBlock::Padding(_) => "a",
					FlacBlock::Picture(_) => "p",
					FlacBlock::SeekTable(_) => "s",
					FlacBlock::VorbisComment(_) => "v",
				}
			);
		}

		return Ok(());
	}
}
