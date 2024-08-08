//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::{
	collections::VecDeque,
	error::Error,
	fmt::Display,
	io::{Cursor, Read, Seek, Write},
};

use super::{
	blocks::{
		FlacAudioFrame, FlacCommentBlock, FlacMetablockDecode, FlacMetablockEncode,
		FlacMetablockHeader, FlacMetablockType,
	},
	errors::{FlacDecodeError, FlacEncodeError},
};
use crate::flac::blocks::{
	FlacApplicationBlock, FlacCuesheetBlock, FlacPaddingBlock, FlacPictureBlock,
	FlacSeektableBlock, FlacStreaminfoBlock,
};

const MIN_AUDIO_FRAME_LEN: usize = 5000;

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

#[derive(Debug)]
#[allow(missing_docs)]
pub enum FlacBlock {
	Streaminfo(FlacStreaminfoBlock),
	Picture(FlacPictureBlock),
	Padding(FlacPaddingBlock),
	Application(FlacApplicationBlock),
	SeekTable(FlacSeektableBlock),
	VorbisComment(FlacCommentBlock),
	CueSheet(FlacCuesheetBlock),
	AudioFrame(FlacAudioFrame),
}

impl FlacBlock {
	/// Encode this block
	pub fn encode(
		&self,
		is_last: bool,
		with_header: bool,
		target: &mut impl Write,
	) -> Result<(), FlacEncodeError> {
		match self {
			Self::Streaminfo(b) => b.encode(is_last, with_header, target),
			Self::SeekTable(b) => b.encode(is_last, with_header, target),
			Self::Picture(b) => b.encode(is_last, with_header, target),
			Self::Padding(b) => b.encode(is_last, with_header, target),
			Self::Application(b) => b.encode(is_last, with_header, target),
			Self::VorbisComment(b) => b.encode(is_last, with_header, target),
			Self::CueSheet(b) => b.encode(is_last, with_header, target),
			Self::AudioFrame(b) => b.encode(target),
		}
	}

	/// Try to decode the given data as a block
	pub fn decode(block_type: FlacMetablockType, data: &[u8]) -> Result<Self, FlacDecodeError> {
		Ok(match block_type {
			FlacMetablockType::Streaminfo => {
				FlacBlock::Streaminfo(FlacStreaminfoBlock::decode(data)?)
			}
			FlacMetablockType::Application => {
				FlacBlock::Application(FlacApplicationBlock::decode(data)?)
			}
			FlacMetablockType::Cuesheet => FlacBlock::CueSheet(FlacCuesheetBlock::decode(data)?),
			FlacMetablockType::Padding => FlacBlock::Padding(FlacPaddingBlock::decode(data)?),
			FlacMetablockType::Picture => FlacBlock::Picture(FlacPictureBlock::decode(data)?),
			FlacMetablockType::Seektable => FlacBlock::SeekTable(FlacSeektableBlock::decode(data)?),
			FlacMetablockType::VorbisComment => {
				FlacBlock::VorbisComment(FlacCommentBlock::decode(data)?)
			}
		})
	}
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
/// This is the foundation of all other flac processors
/// we offer in this crate.
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

	/// If true, this reader has received all the data it needs.
	pub fn is_done(&self) -> bool {
		self.current_block.is_none()
	}

	/// If true, this reader has at least one block ready to pop.
	/// Calling `pop_block` will return `Some(_)` if this is true.
	pub fn has_block(&self) -> bool {
		!self.output_blocks.is_empty()
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

					if data.len() == usize::try_from(header.length).unwrap() {
						// If we picked this block type, add it to the queue
						if self.selector.should_pick_meta(header.block_type) {
							let b = FlacBlock::decode(header.block_type, data)?;
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
					if data.len() - last_read_size <= 2
						&& !(data[0] == 0b1111_1111 && data[1] & 0b1111_1100 == 0b1111_1000)
					{
						return Err(FlacDecodeError::BadSyncBytes.into());
					}

					if data.len() >= MIN_AUDIO_FRAME_LEN {
						// Look for a frame sync header in the data we read
						//
						// This isn't the *correct* way to split audio frames (false sync bytes can occur in audio data),
						// but it's good enough for now---we don't decode audio data anyway.
						//
						// We could split on every sequence of sync bytes, but that's not any less wrong than the approach here.
						// Also, it's slower---we'd rather have few large frames than many small ones.

						let first_byte = if data.len() - last_read_size < MIN_AUDIO_FRAME_LEN {
							MIN_AUDIO_FRAME_LEN + 1
						} else {
							data.len() - last_read_size + MIN_AUDIO_FRAME_LEN + 1
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
									self.output_blocks.push_back(FlacBlock::AudioFrame(
										FlacAudioFrame::decode(&data[0..i - 2])?,
									));
								}

								// Backtrack to the first bit AFTER this new sync sequence
								buf.seek(std::io::SeekFrom::Current(
									-i64::try_from(data.len() - i).unwrap(),
								))
								.unwrap();

								self.current_block = Some(FlacBlockType::AudioData {
									data: {
										let mut v = Vec::with_capacity(MIN_AUDIO_FRAME_LEN);
										v.extend(&data[i - 2..i]);
										v
									},
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
					self.output_blocks
						.push_back(FlacBlock::AudioFrame(FlacAudioFrame::decode(&data)?));
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
	use std::{io::Write, ops::Range, str::FromStr};

	use super::*;
	use crate::{
		common::tagtype::TagType,
		flac::tests::{FlacBlockOutput, FlacTestCase, VorbisCommentTestValue, MANIFEST},
	};

	fn read_file(
		test_case: &FlacTestCase,
		fragment_size_range: Option<Range<usize>>,
		selector: FlacBlockSelector,
	) -> Result<Vec<FlacBlock>, FlacBlockReaderError> {
		let file_data = std::fs::read(test_case.get_path()).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		assert_eq!(test_case.get_in_hash(), format!("{:x}", hasher.finalize()));

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
				reader.push_data(&file_data[head..head + frag_size])?;
				head += frag_size;
			}
		} else {
			reader.push_data(&file_data)?;
		}

		reader.finish()?;
		while let Some(b) = reader.pop_block() {
			out_blocks.push(b)
		}

		return Ok(out_blocks);
	}

	fn test_identical(
		test_case: &FlacTestCase,
		fragment_size_range: Option<Range<usize>>,
	) -> Result<(), FlacBlockReaderError> {
		let out_blocks = read_file(
			test_case,
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
		)?;

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

			b.encode(is_last, true, &mut out).unwrap();
		}

		let mut hasher = Sha256::new();
		hasher.update(out);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, test_case.get_in_hash(), "Output hash doesn't match");
		return Ok(());
	}

	fn test_blockread(
		test_case: &FlacTestCase,
		fragment_size_range: Option<Range<usize>>,
	) -> Result<(), FlacBlockReaderError> {
		let out_blocks = read_file(
			test_case,
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
		)?;

		assert_eq!(
			test_case.get_blocks().unwrap().len(),
			out_blocks
				.iter()
				.filter(|x| !matches!(*x, FlacBlock::AudioFrame(_)))
				.count(),
			"Number of blocks didn't match"
		);

		let mut audio_data_hasher = Sha256::new();
		let mut result_i = 0;

		for b in out_blocks {
			match b {
				FlacBlock::Streaminfo(s) => match &test_case.get_blocks().unwrap()[result_i] {
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

				FlacBlock::Application(a) => match &test_case.get_blocks().unwrap()[result_i] {
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

				FlacBlock::CueSheet(c) => match &test_case.get_blocks().unwrap()[result_i] {
					FlacBlockOutput::CueSheet { hash } => {
						assert_eq!(*hash, {
							let mut hasher = Sha256::new();
							hasher.update(&c.data);
							format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Padding(p) => match &test_case.get_blocks().unwrap()[result_i] {
					FlacBlockOutput::Padding { size } => {
						assert_eq!(p.size, *size);
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::SeekTable(t) => match &test_case.get_blocks().unwrap()[result_i] {
					FlacBlockOutput::Seektable { hash } => {
						assert_eq!(*hash, {
							let mut hasher = Sha256::new();
							hasher.update(&t.data);
							format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Picture(p) => match &test_case.get_blocks().unwrap()[result_i] {
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

				FlacBlock::VorbisComment(v) => match &test_case.get_blocks().unwrap()[result_i] {
					FlacBlockOutput::VorbisComment {
						vendor,
						comments,
						pictures,
					} => {
						assert_eq!(*vendor, v.comment.vendor, "Comment vendor doesn't match");

						assert_eq!(
							v.comment.pictures.len(),
							pictures.len(),
							"Number of pictures doesn't match"
						);

						for (p, e) in v.comment.pictures.iter().zip(*pictures) {
							match e {
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
								_ => panic!("Bad test data: expected only Picture blocks."),
							}
						}

						match comments {
							VorbisCommentTestValue::Raw { tags } => {
								assert_eq!(
									v.comment.comments.len(),
									tags.len(),
									"Number of comments doesn't match"
								);

								for ((got_tag, got_val), (exp_tag, exp_val)) in
									v.comment.comments.iter().zip(*tags)
								{
									assert_eq!(
										*got_tag,
										TagType::from_str(*exp_tag).unwrap(),
										"Tag key doesn't match"
									);
									assert_eq!(
										got_val, exp_val,
										"Tag value of {exp_tag} doesn't match"
									);
								}
							}

							VorbisCommentTestValue::Hash { n_comments, hash } => {
								assert_eq!(
									v.comment.comments.len(),
									*n_comments,
									"Number of comments doesn't match"
								);

								let mut hasher = Sha256::new();
								for (got_tag, got_val) in v.comment.comments {
									hasher.update(format!("{got_tag}={got_val};").as_bytes())
								}
								assert_eq!(
									&format!("{:x}", hasher.finalize()),
									hash,
									"Comment hash doesn't match"
								);
							}
						}
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::AudioFrame(data) => {
					data.encode(&mut audio_data_hasher).unwrap();

					if result_i != test_case.get_blocks().unwrap().len() {
						panic!("There are metadata blocks between audio frames!")
					}

					// Don't increment result_i
					continue;
				}
			}

			result_i += 1;
		}

		// Check audio data hash
		assert_eq!(
			test_case.get_audio_hash().unwrap(),
			format!("{:x}", audio_data_hasher.finalize())
		);

		return Ok(());
	}

	// Helper macros to generate tests
	macro_rules! gen_tests {
		( $test_name:ident ) => {
			paste! {
				#[test]
				pub fn [<blockread_small_ $test_name>]() {
					let test_case = MANIFEST.iter().find(|x| x.get_name() == stringify!($test_name)).unwrap();

					match test_case {
						FlacTestCase::Success { .. } => {
							for _ in 0..5 {
								test_blockread(
									test_case,
									Some(1..256),
								).unwrap()
							}
						},

						FlacTestCase::Error { check_error, .. } => {
							let e = test_blockread(test_case, Some(1..256)).unwrap_err();
							match e {
								FlacBlockReaderError::DecodeError(e) => assert!(check_error(&e), "Unexpected error {e:?}"),
								_ => panic!("Unexpected error {e:?}")
							}
						}
					}
				}

				#[test]
				pub fn [<identical_small_ $test_name>]() {
					let test_case = MANIFEST.iter().find(|x| x.get_name() == stringify!($test_name)).unwrap();

					match test_case {
						FlacTestCase::Success { .. } => {
							for _ in 0..5 {
								test_identical(
									test_case,
									Some(1..256),
								).unwrap()
							}
						},

						FlacTestCase::Error { check_error, .. } => {
							let e = test_identical(test_case, Some(1..256)).unwrap_err();
							match e {
								FlacBlockReaderError::DecodeError(e) => assert!(check_error(&e), "Unexpected error {e:?}"),
								_ => panic!("Unexpected error {e:?}")
							}
						}
					}
				}
			}
		};
	}

	gen_tests!(custom_01);
	gen_tests!(custom_02);
	gen_tests!(custom_03);

	gen_tests!(uncommon_10);

	gen_tests!(faulty_06);
	gen_tests!(faulty_07);
	gen_tests!(faulty_10);
	gen_tests!(faulty_11);

	gen_tests!(subset_45);
	gen_tests!(subset_46);
	gen_tests!(subset_47);
	gen_tests!(subset_48);
	gen_tests!(subset_49);
	gen_tests!(subset_50);
	gen_tests!(subset_51);
	gen_tests!(subset_52);
	gen_tests!(subset_53);
	gen_tests!(subset_54);
	gen_tests!(subset_55);
	gen_tests!(subset_56);
	gen_tests!(subset_57);
	gen_tests!(subset_58);
	gen_tests!(subset_59);
}
