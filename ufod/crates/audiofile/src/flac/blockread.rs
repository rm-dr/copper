//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::{
	collections::VecDeque,
	error::Error,
	fmt::Display,
	io::{Cursor, Read, Seek},
};

use super::{
	blocks::{
		FlacAudioFrame, FlacCommentBlock, FlacMetablockDecode, FlacMetablockHeader,
		FlacMetablockType,
	},
	errors::FlacDecodeError,
};
use crate::flac::blocks::{
	FlacApplicationBlock, FlacCuesheetBlock, FlacPaddingBlock, FlacPictureBlock,
	FlacSeektableBlock, FlacStreaminfoBlock,
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
	VorbisComment(FlacCommentBlock),
	CueSheet(FlacCuesheetBlock),
	AudioFrame(Vec<u8>),
}

/// An error produced by a [`FlacBlockReader`]
#[derive(Debug)]
pub enum FlacBlockReaderError {
	/// Could not decode flac data
	DecodeError(FlacDecodeError),

	/// Tried to finish or push data to a finished reader.
	AlreadyFinished,
}

impl Display for FlacBlockReaderError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DecodeError(_) => write!(f, "decode error while reading flac blocks"),
			Self::AlreadyFinished => write!(f, "flac block reader is already finished"),
		}
	}
}

impl Error for FlacBlockReaderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			Self::DecodeError(e) => e,
			Self::AlreadyFinished => return None,
		})
	}
}

impl From<FlacDecodeError> for FlacBlockReaderError {
	fn from(value: FlacDecodeError) -> Self {
		Self::DecodeError(value)
	}
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
	pub fn push_data(&mut self, buf: &[u8]) -> Result<(), FlacBlockReaderError> {
		let mut buf = Cursor::new(buf);
		let mut last_read_size = 1;

		if self.current_block.is_none() {
			return Err(FlacBlockReaderError::AlreadyFinished);
		}

		'outer: while last_read_size != 0 {
			match self.current_block.as_mut().unwrap() {
				FlacBlockType::MagicBits { data, left_to_read } => {
					last_read_size = buf.read(&mut data[4 - *left_to_read..4]).unwrap();
					*left_to_read -= last_read_size;

					if *left_to_read == 0 {
						if *data != [0x66, 0x4C, 0x61, 0x43] {
							return Err(FlacDecodeError::BadMagicBytes.into());
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
							return Err(FlacDecodeError::BadFirstBlock.into());
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
									FlacBlock::VorbisComment(FlacCommentBlock::decode(&data)?)
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
							return Err(FlacDecodeError::BadSyncBytes.into());
						}
					}

					if data.len() > 2 {
						// Look for a frame sync header in the data we read

						let first_byte = if data.len() - last_read_size < 2 {
							2
						} else {
							(data.len() - last_read_size) + 2
						};

						// `i` is the index of the first byte *after* the sync sequence.
						//
						// This may seem odd, but it makes the odd edge case easier to handle:
						// If we instead have `i` be the index of the first byte *of* the frame sequence,
						// dealing with the case where `data` contained half the sync sequence before
						// reading is tricky.
						for i in first_byte..data.len() {
							if data[i - 2] == 0b1111_1111
								&& data[i - 1] & 0b1111_1100 == 0b1111_1000
							{
								// We found another frame sync header. Split at this index.
								if self.selector.pick_audio {
									self.output_blocks
										.push_back(FlacBlock::AudioFrame(Vec::from(
											&data[0..i - 2],
										)));
								}

								// Backtrack to the first bit AFTER this new sync sequence
								buf.seek(std::io::SeekFrom::Current(
									-i64::try_from(data.len() - i).unwrap(),
								))
								.unwrap();

								self.current_block = Some(FlacBlockType::AudioData {
									data: Vec::from(&data[i - 2..i]),
								});
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
	///
	/// `finish()` should be called exactly once once we have finished each stream.
	/// Finishing twice or pushing data to a finished reader results in a panic.
	pub fn finish(&mut self) -> Result<(), FlacBlockReaderError> {
		match self.current_block.take() {
			None => return Err(FlacBlockReaderError::AlreadyFinished),

			Some(FlacBlockType::AudioData { data }) => {
				// We can't run checks if we don't have enough data.
				if data.len() <= 2 {
					return Err(FlacDecodeError::MalformedBlock.into());
				}

				if !(data[0] == 0b1111_1111 && data[1] & 0b1111_1100 == 0b1111_1000) {
					return Err(FlacDecodeError::BadSyncBytes.into());
				}

				if self.selector.pick_audio {
					self.output_blocks.push_back(FlacBlock::AudioFrame(data));
				}

				self.current_block = None;
				return Ok(());
			}

			// All other blocks have a known length and
			// are finished automatically.
			_ => return Err(FlacDecodeError::MalformedBlock.into()),
		}
	}
}

#[cfg(test)]
mod tests {
	use itertools::Itertools;
	use paste::paste;
	use rand::Rng;
	use sha2::{Digest, Sha256};
	use std::{
		io::Write,
		path::{Path, PathBuf},
	};
	use ufo_util::mime::MimeType;

	use super::*;
	use crate::{common::picturetype::PictureType, flac::blocks::FlacMetablockEncode};

	enum FlacBlockOutput {
		Application {
			application_id: u32,
			hash: &'static str,
		},
		Streaminfo {
			min_block_size: u32,
			max_block_size: u32,
			min_frame_size: u32,
			max_frame_size: u32,
			sample_rate: u32,
			channels: u8,
			bits_per_sample: u8,
			total_samples: u128,
			md5_signature: &'static str,
		},
		CueSheet {
			// Hash of this block's data, without the header.
			// This is easy to get with
			//
			// ```notrust
			// metaflac \
			//	--list \
			//	--block-number=<n> \
			//	--data-format=binary-headerless \
			//	<file> \
			//	| sha256sum
			//```
			hash: &'static str,
		},
		Seektable {
			hash: &'static str,
		},
		Padding {
			size: u32,
		},
		Picture {
			picture_type: PictureType,
			mime: MimeType,
			description: &'static str,
			width: u32,
			height: u32,
			bit_depth: u32,
			color_count: u32,
			img_data: &'static str,
		},
		VorbisComment {
			hash: &'static str,
		},
	}

	fn read_file(
		test_file_path: &Path,
		fragment_size_range: Option<std::ops::Range<usize>>,

		selector: FlacBlockSelector,
		in_hash: &str,
	) -> Vec<FlacBlock> {
		let file_data = std::fs::read(test_file_path).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		assert_eq!(in_hash, format!("{:x}", hasher.finalize()));

		let mut reader = FlacBlockReader::new(selector);
		let mut out_blocks = Vec::new();

		// Push file data to the reader, in parts or as a whole.
		if let Some(fragment_size_range) = fragment_size_range {
			let mut head = 0;
			while head < file_data.len() {
				let mut frag_size = rand::thread_rng().gen_range(fragment_size_range.clone());
				if head + frag_size > file_data.len() {
					frag_size = file_data.len() - head;
				}
				reader
					.push_data(&file_data[head..head + frag_size])
					.unwrap();
				head += frag_size;
			}
		} else {
			reader.push_data(&file_data).unwrap();
		}

		reader.finish().unwrap();
		while let Some(b) = reader.pop_block() {
			out_blocks.push(b)
		}

		return out_blocks;
	}

	fn test_strip(
		test_file_path: &Path,
		fragment_size_range: Option<std::ops::Range<usize>>,
		in_hash: &str,
		out_hash: &str,
	) {
		let out_blocks = read_file(
			test_file_path,
			fragment_size_range,
			FlacBlockSelector {
				pick_streaminfo: true,
				pick_padding: false,
				pick_application: false,
				pick_seektable: true,
				pick_vorbiscomment: false,
				pick_cuesheet: true,
				pick_picture: false,
				pick_audio: true,
			},
			in_hash,
		);

		let mut out = Vec::new();
		out.write_all(&[0x66, 0x4C, 0x61, 0x43]).unwrap();

		for i in 0..out_blocks.len() {
			let b = &out_blocks[i];
			let is_last = if i == out_blocks.len() - 1 {
				false
			} else {
				!matches!(b, FlacBlock::AudioFrame(_))
					&& matches!(&out_blocks[i + 1], FlacBlock::AudioFrame(_))
			};

			match b {
				FlacBlock::Streaminfo(i) => i.encode(is_last, &mut out).unwrap(),
				FlacBlock::CueSheet(c) => c.encode(is_last, &mut out).unwrap(),
				FlacBlock::SeekTable(s) => s.encode(is_last, &mut out).unwrap(),
				FlacBlock::AudioFrame(a) => out.extend(a),
				_ => unreachable!(),
			}
		}

		let mut hasher = Sha256::new();
		hasher.update(out);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, out_hash, "Stripped FLAC hash doesn't match");
	}

	fn test_blockread(
		test_file_path: &Path,
		fragment_size_range: Option<std::ops::Range<usize>>,
		in_hash: &str,
		result: &[FlacBlockOutput],
		audio_hash: &str,
	) {
		let out_blocks = read_file(
			test_file_path,
			fragment_size_range,
			FlacBlockSelector {
				pick_streaminfo: true,
				pick_padding: true,
				pick_application: true,
				pick_seektable: true,
				pick_vorbiscomment: true,
				pick_cuesheet: true,
				pick_picture: true,
				pick_audio: true,
			},
			in_hash,
		);

		assert_eq!(
			result.len(),
			out_blocks
				.iter()
				.filter(|x| !matches!(*x, FlacBlock::AudioFrame(_)))
				.count(),
			"Number of blocks didn't match"
		);

		println!("{:?}", out_blocks.len());

		let mut audio_data_hasher = Sha256::new();
		let mut result_i = 0;

		for b in out_blocks {
			match b {
				FlacBlock::Streaminfo(s) => match &result[result_i] {
					FlacBlockOutput::Streaminfo {
						min_block_size,
						max_block_size,
						min_frame_size,
						max_frame_size,
						sample_rate,
						channels,
						bits_per_sample,
						total_samples,
						md5_signature,
					} => {
						assert_eq!(*min_block_size, s.min_block_size,);
						assert_eq!(*max_block_size, s.max_block_size);
						assert_eq!(*min_frame_size, s.min_frame_size);
						assert_eq!(*max_frame_size, s.max_frame_size);
						assert_eq!(*sample_rate, s.sample_rate);
						assert_eq!(*channels, s.channels);
						assert_eq!(*bits_per_sample, s.bits_per_sample);
						assert_eq!(*total_samples, s.total_samples);
						assert_eq!(
							*md5_signature,
							s.md5_signature.iter().map(|x| format!("{x:02x}")).join("")
						);
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Application(a) => match &result[result_i] {
					FlacBlockOutput::Application {
						application_id,
						hash,
					} => {
						assert_eq!(
							*application_id, a.application_id,
							"Application id doesn't match"
						);
						assert_eq!(
							*hash,
							{
								let mut hasher = Sha256::new();
								hasher.update(&a.data);
								format!("{:x}", hasher.finalize())
							},
							"Application content hash doesn't match"
						);
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::CueSheet(c) => match &result[result_i] {
					FlacBlockOutput::CueSheet { hash } => {
						assert_eq!(*hash, {
							let mut hasher = Sha256::new();
							hasher.update(&c.data);
							format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Padding(p) => match &result[result_i] {
					FlacBlockOutput::Padding { size } => {
						assert_eq!(*size, p.size.try_into().unwrap());
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::SeekTable(t) => match &result[result_i] {
					FlacBlockOutput::Seektable { hash } => {
						assert_eq!(*hash, {
							let mut hasher = Sha256::new();
							hasher.update(&t.data);
							format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Picture(p) => match &result[result_i] {
					FlacBlockOutput::Picture {
						picture_type,
						mime,
						description,
						width,
						height,
						bit_depth,
						color_count,
						img_data,
					} => {
						assert_eq!(*picture_type, p.picture_type);
						assert_eq!(*mime, p.mime);
						assert_eq!(*description, p.description);
						assert_eq!(*width, p.width);
						assert_eq!(*height, p.height);
						assert_eq!(*bit_depth, p.bit_depth);
						assert_eq!(*color_count, p.color_count);
						assert_eq!(*img_data, {
							let mut hasher = Sha256::new();
							hasher.update(&p.img_data);
							&format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::VorbisComment(_) => {}

				FlacBlock::AudioFrame(data) => {
					audio_data_hasher.update(data);

					if result_i != result.len() {
						panic!("There are metadata blocks betwen audio frames!")
					}

					// Don't increment result_i
					continue;
				}
			}

			result_i += 1;
		}

		// Check audio data hash
		assert_eq!(audio_hash, format!("{:x}", audio_data_hasher.finalize()));
	}

	// Helper macros to generate tests
	macro_rules! test_success {
		(
					// The name of this test
					$file_name:ident,

					// The path to the test file
					$file_path:expr,

					// SHA-256 hash of unmodified source file
					$in_hash:literal,

					// The blocks we expect to find
					$result:expr,

					// The expected hash of audio data
					//
					// Get this hash by running `metaflac --remove-all --dont-use-padding`,
					// then by manually deleting remaining headers in a hex editor
					// (Remember that the sync sequence is 0xFF 0xF8)
					$audio_hash:literal,

					// The hash of this file with tags stripped
					$stripped_hash:literal
				) => {
			paste! {
			#[test]
			pub fn [<blockread_small_ $file_name>]() {
				for _ in 0..5 {
					test_blockread(
						$file_path,
						Some(1..256),
						$in_hash,
						$result,
						$audio_hash
					)
				}
			}

			#[test]
			pub fn [<blockread_large_ $file_name>]() {
				for _ in 0..5 {
					test_blockread(
						$file_path,
						Some(5_000..100_000),
						$in_hash,
						$result,
						$audio_hash
					)
				}
			}

			#[test]
			pub fn [<blockread_strip_ $file_name>]() {
				for _ in 0..5 {
					test_strip(
						$file_path,
						Some(5_000..100_000),
						$in_hash,
						$stripped_hash
					)
				}
			}
				}
		};
	}

	test_success!(
		subset_45,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/45 - no total number of samples set.flac"),
		"336a18eb7a78f7fc0ab34980348e2895bc3f82db440a2430d9f92e996f889f9a",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 907,
				max_frame_size: 8053,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 0,
				md5_signature: "c41ae3b82c35d8f5c3dab1729f948fde"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" }
		],
		"3fb3482ebc1724559bdd57f34de472458563d78a676029614e76e32b5d2b8816",
		"31631ac227ebe2689bac7caa1fa964b47e71a9f1c9c583a04ea8ebd9371508d0"
	);

	test_success!(
		subset_46,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/46 - no min-max framesize set.flac"),
		"9dc39732ce17815832790901b768bb50cd5ff0cd21b28a123c1cabc16ed776cc",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 0,
				max_frame_size: 0,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 282866,
				md5_signature: "fd131e6ebc75251ed83f8f4c07df36a4"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" }
		],
		"a1eed422462b386a932b9eb3dff3aea3687b41eca919624fb574aadb7eb50040",
		"9e57cd77f285fc31f87fa4e3a31ab8395d68d5482e174c8e0d0bba9a0c20ba27"
	);

	test_success!(
		subset_47,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/47 - only STREAMINFO.flac"),
		"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
		&[FlacBlockOutput::Streaminfo {
			min_block_size: 4096,
			max_block_size: 4096,
			min_frame_size: 4747,
			max_frame_size: 7034,
			sample_rate: 48000,
			channels: 2,
			bits_per_sample: 16,
			total_samples: 232608,
			md5_signature: "bba30c5f70789910e404b7ac727c3853"
		},],
		"5ee1450058254087f58c91baf0f70d14bde8782cf2dc23c741272177fe0fce6e",
		"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85"
	);

	test_success!(
		subset_48,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/48 - Extremely large SEEKTABLE.flac"),
		"4417aca6b5f90971c50c28766d2f32b3acaa7f9f9667bd313336242dae8b2531",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 2445,
				max_frame_size: 7364,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 259884,
				md5_signature: "97a0574290237563fbaa788ad77d2cdf"
			},
			FlacBlockOutput::Seektable {
				hash: "21ca2184ae22fe26b690fd7cbd8d25fcde1d830ff6e5796ced4107bab219d7c0"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" }
		],
		"c2d691f2c4c986fe3cd5fd7864d9ba9ce6dd68a4ffc670447f008434b13102c2",
		"abc9a0c40a29c896bc6e1cc0b374db1c8e157af716a5a3c43b7db1591a74c4e8"
	);

	test_success!(
		subset_49,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/49 - Extremely large PADDING.flac"),
		"7bc44fa2754536279fde4f8fb31d824f43b8d0b3f93d27d055d209682914f20e",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 1353,
				max_frame_size: 7117,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 258939,
				md5_signature: "6e78f221caaaa5d570a53f1714d84ded"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Padding { size: 16777215 }
		],
		"5007be7109b28b0149d1b929d2a0e93a087381bd3e68cf2a3ef78ea265ea20c3",
		"a2283bbacbc4905ad3df1bf9f43a0ea7aa65cf69523d84a7dd8eb54553cc437e"
	);

	test_success!(
		subset_50,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/50 - Extremely large PICTURE.flac"),
		"1f04f237d74836104993a8072d4223e84a5d3bd76fbc44555c221c7e69a23594",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 5099,
				max_frame_size: 7126,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 265617,
				md5_signature: "82164e4da30ed43b47e6027cef050648"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 3200,
				height: 2252,
				bit_depth: 24,
				color_count: 0,
				img_data: "b78c3a48fde4ebbe8e4090e544caeb8f81ed10020d57cc50b3265f9b338d8563",
			},
		],
		"9778b25c5d1f56cfcd418e550baed14f9d6a4baf29489a83ed450fbebb28de8c",
		"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c"
	);

	test_success!(
		subset_51,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/51 - Extremely large VORBISCOMMENT.flac"),
		"033160e8124ed287b0b5d615c94ac4139477e47d6e4059b1c19b7141566f5ef9",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 4531,
				max_frame_size: 7528,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 289972,
				md5_signature: "5ff622c88f8dd9bc201a6a541f3890d3"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" }
		],
		"76419865d10eb22a74f020423a4e515e800f0177441676afd0418557c2d76c36",
		"c0ca6c6099b5d9ec53d6bb370f339b2b1570055813a6cd3616fac2db83a2185e"
	);

	test_success!(
		subset_52,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/52 - Extremely large APPLICATION.flac"),
		"0e45a4f8dbef15cbebdd8dfe690d8ae60e0c6abb596db1270a9161b62a7a3f1c",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 3711,
				max_frame_size: 7056,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 317876,
				md5_signature: "eb7140266bc194527488c21ab49bc47b"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Application {
				application_id: 0x74657374,
				hash: "cfc0b8969e4ba6bd507999ba89dea2d274df69d94749d6ae3cf117a7780bba09"
			}
		],
		"89ad1a5c86a9ef35d33189c81c8a90285a23964a13f8325bf2c02043e8c83d63",
		"cc4a0afb95ec9bcde8ee33f13951e494dc4126a9a3a668d79c80ce3c14a3acd9"
	);

	test_success!(
		subset_53,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/53 - CUESHEET with very many indexes.flac"),
		"513fad18578f3225fae5de1bda8f700415be6fd8aa1e7af533b5eb796ed2d461",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 2798,
				max_frame_size: 7408,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 2910025,
				md5_signature: "d11f3717d628cfe6a90a10facc478340"
			},
			FlacBlockOutput::Seektable {
				hash: "18629e1b874cb27e4364da72fb3fec2141eb0618baae4a1cee6ed09562aa00a8"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::CueSheet {
				hash: "70638a241ca06881a52c0a18258ea2d8946a830137a70479c49746d2a1344bdd"
			},
		],
		"e993070f2080f2c598be1d61d208e9187a55ddea4be1d2ed1f8043e7c03e97a5",
		"57c5b945e14c6fcd06916d6a57e5b036d67ff35757893c24ed872007aabbcf4b"
	);

	test_success!(
		subset_54,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/54 - 1000x repeating VORBISCOMMENT.flac"),
		"b68dc6644784fac35aa07581be8603a360d1697e07a2265d7eb24001936fd247",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 1694,
				max_frame_size: 7145,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 433151,
				md5_signature: "1d950e92b357dedbc5290a7f2210a2ef"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" }
		],
		"4721b784058410c6263f73680079e9a71aee914c499afcf5580c121fce00e874",
		"5c8b92b83c0fa17821add38263fa323d1c66cfd2ee57aca054b50bd05b9df5c2"
	);

	test_success!(
		subset_55,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/55 - file 48-53 combined.flac"),
		"a756b460df79b7cc492223f80cda570e4511f2024e5fa0c4d505ba51b86191f6",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 3103,
				max_frame_size: 11306,
				sample_rate: 44100,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 2646000,
				md5_signature: "2c78978cbbff11daac296fee97c3e061"
			},
			FlacBlockOutput::Seektable {
				hash: "58dfa7bac4974edf1956b068f5aa72d1fbd9301c36a3085a8a57b9db11a2dbf0"
			},
			FlacBlockOutput::VorbisComment { hash: "todo" },
			FlacBlockOutput::CueSheet {
				hash: "db11916c8f5f39648256f93f202e00ff8d73d7d96b62f749b4c77cf3ea744f90"
			},
			FlacBlockOutput::Application {
				application_id: 0x74657374,
				hash: "6088a557a1bad7bfa5ebf79a324669fbf4fa2f8e708f5487305dfc5b2ff2249a"
			},
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 3200,
				height: 2252,
				bit_depth: 24,
				color_count: 0,
				img_data: "b78c3a48fde4ebbe8e4090e544caeb8f81ed10020d57cc50b3265f9b338d8563",
			},
			FlacBlockOutput::Padding { size: 16777215 }
		],
		"f1285b77cec7fa9a0979033244489a9d06b8515b2158e9270087a65a4007084d",
		"401038fce06aff5ebdc7a5f2fc01fa491cbf32d5da9ec99086e414b2da3f8449"
	);

	test_success!(
		subset_56,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/56 - JPG PICTURE.flac"),
		"5cebe7a3710cf8924bd2913854e9ca60b4cd53cfee5a3af0c3c73fddc1888963",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 3014,
				max_frame_size: 7219,
				sample_rate: 44100,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 220026,
				md5_signature: "5b0e898d9c2626d0c28684f5a586813f"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 1920,
				height: 1080,
				bit_depth: 24,
				color_count: 0,
				img_data: "7a3ed658f80f433eee3914fff451ea0312807de0af709e37cc6a4f3f6e8a47c6",
			},
		],
		"ccfe90b0f15cd9662f7a18f40cd4c347538cf8897a08228e75351206f7804573",
		"31a38d59db2010790b7abf65ec0cc03f2bbe1fed5952bc72bee4ca4d0c92e79f"
	);

	test_success!(
		subset_57,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/57 - PNG PICTURE.flac"),
		"c6abff7f8bb63c2821bd21dd9052c543f10ba0be878e83cb419c248f14f72697",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 463,
				max_frame_size: 6770,
				sample_rate: 44100,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 221623,
				md5_signature: "ad16957bcf8d5a3ec8caf261e43d5ff7"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Png,
				description: "",
				width: 960,
				height: 540,
				bit_depth: 24,
				color_count: 0,
				img_data: "d804e5c7b9ee5af694b5e301c6cdf64508ff85997deda49d2250a06a964f10b2",
			},
		],
		"39bf9981613ac2f35d253c0c21b76a48abba7792c27da5dbf23e6021e2e6673f",
		"3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1"
	);

	test_success!(
		subset_58,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/58 - GIF PICTURE.flac"),
		"7c2b1a963a665847167a7275f9924f65baeb85c21726c218f61bf3f803f301c8",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 2853,
				max_frame_size: 6683,
				sample_rate: 44100,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 219826,
				md5_signature: "7c1810602a7db96d7a48022ac4aa495c"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Unknown("image/gif".into()),
				description: "",
				width: 1920,
				height: 1080,
				bit_depth: 24,
				color_count: 32,
				img_data: "e33cccc1d799eb2bb618f47be7099cf02796df5519f3f0e1cc258606cf6e8bb1",
			},
		],
		"30e3292e9f56cf88658eeadfdec8ad3a440690ce6d813e1b3374f60518c8e0ae",
		"4cd771e27870e2a586000f5b369e0426183a521b61212302a2f5802b046910b2"
	);

	test_success!(
		subset_59,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/59 - AVIF PICTURE.flac"),
		"7395d02bf8d9533dc554cce02dee9de98c77f8731a45f62d0a243bd0d6f9a45c",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 153,
				max_frame_size: 7041,
				sample_rate: 44100,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 221423,
				md5_signature: "d354246011ca204159c06f52cad5f634"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Unknown("image/avif".into()),
				description: "",
				width: 1920,
				height: 1080,
				bit_depth: 24,
				color_count: 0,
				img_data: "a431123040c74f75096237f20544a7fb56b4eb71ddea62efa700b0a016f5b2fc",
			},
		],
		"b208c73d274e65b27232bfffbfcbcf4805ee3cbc9cfbf7d2104db8f53370273b",
		"d5215e16c6b978fc2c3e6809e1e78981497cb8514df297c5169f3b4a28fd875c"
	);

	test_success!(
		custom_01,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_custom/01 - many images.flac"),
		"58ee39efe51e37f51b4dedeee8b28bed88ac1d4a70ba0e3a326ef7e94f0ebf1b",
		&[
			FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 5099,
				max_frame_size: 7126,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 265617,
				md5_signature: "82164e4da30ed43b47e6027cef050648"
			},
			FlacBlockOutput::VorbisComment { hash: "idk" },
			FlacBlockOutput::Picture {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 3200,
				height: 2252,
				bit_depth: 24,
				color_count: 0,
				img_data: "b78c3a48fde4ebbe8e4090e544caeb8f81ed10020d57cc50b3265f9b338d8563",
			},
			FlacBlockOutput::Picture {
				picture_type: PictureType::ABrightColoredFish,
				mime: MimeType::Jpg,
				description: "lorem",
				width: 1920,
				height: 1080,
				bit_depth: 24,
				color_count: 0,
				img_data: "7a3ed658f80f433eee3914fff451ea0312807de0af709e37cc6a4f3f6e8a47c6",
			},
			FlacBlockOutput::Picture {
				picture_type: PictureType::OtherFileIcon,
				mime: MimeType::Png,
				description: "ipsum",
				width: 960,
				height: 540,
				bit_depth: 24,
				color_count: 0,
				img_data: "d804e5c7b9ee5af694b5e301c6cdf64508ff85997deda49d2250a06a964f10b2",
			},
			FlacBlockOutput::Picture {
				picture_type: PictureType::Lyricist,
				mime: MimeType::from("image/gif"),
				description: "dolor",
				width: 1920,
				height: 1080,
				bit_depth: 24,
				color_count: 32,
				img_data: "e33cccc1d799eb2bb618f47be7099cf02796df5519f3f0e1cc258606cf6e8bb1",
			},
			FlacBlockOutput::Picture {
				picture_type: PictureType::BackCover,
				mime: MimeType::from("image/avif"),
				description: "est",
				width: 1920,
				height: 1080,
				bit_depth: 24,
				color_count: 0,
				img_data: "a431123040c74f75096237f20544a7fb56b4eb71ddea62efa700b0a016f5b2fc",
			}
		],
		"9778b25c5d1f56cfcd418e550baed14f9d6a4baf29489a83ed450fbebb28de8c",
		"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c"
	);
}
