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
	use itertools::Itertools;
	use paste::paste;
	use sha2::{Digest, Sha256};
	use std::path::{Path, PathBuf};
	use ufo_util::mime::MimeType;

	use super::*;
	use crate::common::picturetype::PictureType;

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

	fn test_block_whole(
		test_file_path: &Path,
		in_hash: &str,
		result: &[FlacBlockOutput],
		audio_hash: &str,
	) -> Result<(), FlacError> {
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

		let file_data = std::fs::read(test_file_path).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		assert_eq!(in_hash, format!("{:x}", hasher.finalize()));

		let mut x = FlacBlockReader::new(selector);

		x.push_data(&file_data)?;
		x.finish()?;

		let mut all_audio_frames = Vec::new();
		let mut out_blocks = Vec::new();
		while let Some(b) = x.pop_block() {
			match b {
				FlacBlock::AudioFrame(f) => {
					all_audio_frames.extend(f);
				}
				_ => out_blocks.push(b),
			}
		}

		assert_eq!(result.len(), out_blocks.len());

		for (b, r) in out_blocks.iter().zip(result.iter()) {
			match b {
				FlacBlock::Streaminfo(s) => match r {
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
						assert_eq!(*min_block_size, s.min_block_size);
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

				FlacBlock::Application(a) => match r {
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

				FlacBlock::CueSheet(c) => match r {
					FlacBlockOutput::CueSheet { hash } => {
						assert_eq!(*hash, {
							let mut hasher = Sha256::new();
							hasher.update(&c.data);
							format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Padding(p) => match r {
					FlacBlockOutput::Padding { size } => {
						assert_eq!(*size, p.size.try_into().unwrap());
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::SeekTable(t) => match r {
					FlacBlockOutput::Seektable { hash } => {
						assert_eq!(*hash, {
							let mut hasher = Sha256::new();
							hasher.update(&t.data);
							format!("{:x}", hasher.finalize())
						});
					}
					_ => panic!("Unexpected block type"),
				},

				FlacBlock::Picture(p) => match r {
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

				FlacBlock::AudioFrame(_) => {
					unreachable!()
				}
			}
		}

		// Check audio data hash
		let mut hasher = Sha256::new();
		hasher.update(all_audio_frames);
		assert_eq!(audio_hash, format!("{:x}", hasher.finalize()));

		return Ok(());
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
					$audio_hash:literal
				) => {
			paste! {
				#[test]
				pub fn [<blockread_ $file_name>]() {
					test_block_whole(
						$file_path,
						$in_hash,
						$result,
						$audio_hash
					).unwrap()
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
		// Get this hash by running `metaflac --remove-all --dont-use-padding`,
		// then by manually deleting remaining headers in a hex editor
		// (Remember that the sync sequence is 0xFF 0xF8)
		"3fb3482ebc1724559bdd57f34de472458563d78a676029614e76e32b5d2b8816"
	);

	// metaflac --list a --block-number=1 --data-format=binary-headerless | sha256sum

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
		"a1eed422462b386a932b9eb3dff3aea3687b41eca919624fb574aadb7eb50040"
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
		"5ee1450058254087f58c91baf0f70d14bde8782cf2dc23c741272177fe0fce6e"
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
		"c2d691f2c4c986fe3cd5fd7864d9ba9ce6dd68a4ffc670447f008434b13102c2"
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
		"5007be7109b28b0149d1b929d2a0e93a087381bd3e68cf2a3ef78ea265ea20c3"
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
		"9778b25c5d1f56cfcd418e550baed14f9d6a4baf29489a83ed450fbebb28de8c"
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
		"76419865d10eb22a74f020423a4e515e800f0177441676afd0418557c2d76c36"
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
		"89ad1a5c86a9ef35d33189c81c8a90285a23964a13f8325bf2c02043e8c83d63"
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
		"e993070f2080f2c598be1d61d208e9187a55ddea4be1d2ed1f8043e7c03e97a5"
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
		"4721b784058410c6263f73680079e9a71aee914c499afcf5580c121fce00e874"
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
		"f1285b77cec7fa9a0979033244489a9d06b8515b2158e9270087a65a4007084d"
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
		"ccfe90b0f15cd9662f7a18f40cd4c347538cf8897a08228e75351206f7804573"
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
		"39bf9981613ac2f35d253c0c21b76a48abba7792c27da5dbf23e6021e2e6673f"
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
		"30e3292e9f56cf88658eeadfdec8ad3a440690ce6d813e1b3374f60518c8e0ae"
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
		"b208c73d274e65b27232bfffbfcbcf4805ee3cbc9cfbf7d2104db8f53370273b"
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
		"9778b25c5d1f56cfcd418e550baed14f9d6a4baf29489a83ed450fbebb28de8c"
	);
}
