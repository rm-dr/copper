//! Parse FLAC metadata.

pub mod blockread;
pub mod blocks;
pub mod errors;
pub mod proc;

#[cfg(test)]
mod tests {

	use itertools::Itertools;
	use copper_util::mime::MimeType;

	use super::errors::FlacDecodeError;
	use crate::common::{picturetype::PictureType, vorbiscomment::VorbisCommentDecodeError};

	/// The value of a vorbis comment.
	///
	/// Some files have VERY large comments, and providing them
	/// explicitly here doesn't make sense.
	pub enum VorbisCommentTestValue {
		/// The comments, in order
		Raw {
			tags: &'static [(&'static str, &'static str)],
		},
		/// The hash of all comments concatenated together,
		/// stringified as `{key}={value};`
		Hash {
			n_comments: usize,
			hash: &'static str,
		},
	}

	pub enum FlacBlockOutput {
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
			vendor: &'static str,
			comments: VorbisCommentTestValue,
			pictures: &'static [FlacBlockOutput],
		},
	}

	pub enum FlacTestCase {
		Success {
			/// This test's name
			test_name: &'static str,

			/// The file to use for this test
			file_path: &'static str,

			/// The hash of the input files
			in_hash: &'static str,

			/// The flac metablocks we expect to find in this file, in order.
			blocks: &'static [FlacBlockOutput],

			/// The hash of the audio frames in this file
			///
			/// Get this hash by running `metaflac --remove-all --dont-use-padding`,
			/// then by manually deleting remaining headers in a hex editor
			/// (Remember that the sync sequence is 0xFF 0xF8)
			audio_hash: &'static str,

			/// The hash we should get when we strip this file's tags.
			///
			/// A stripped flac file has unmodified STREAMINFO, SEEKTABLE,
			/// CUESHEET, and audio data blocks; and nothing else (not even padding).
			///
			/// Reference implementation:
			/// ```notrust
			/// metaflac \
			/// 	--remove \
			/// 	--block-type=PADDING,APPLICATION,VORBIS_COMMENT,PICTURE \
			/// 	--dont-use-padding \
			/// 	<file>
			/// ```
			stripped_hash: &'static str,
		},
		Error {
			/// This test's name
			test_name: &'static str,

			/// The file to use for this test
			file_path: &'static str,

			/// The hash of the input files
			in_hash: &'static str,

			/// The error we should encounter while reading this file
			check_error: &'static dyn Fn(&FlacDecodeError) -> bool,

			/// If some, stripping this file's metadata should produce the given hash.
			/// If none, trying to strip metadata should produce `check_error`
			stripped_hash: Option<&'static str>,

			/// If some, the following images should be extracted from this file
			/// If none, trying to strip images should produce `check_error`
			pictures: Option<&'static [FlacBlockOutput]>,
		},
	}

	impl FlacTestCase {
		pub fn get_name(&self) -> &str {
			match self {
				Self::Error { test_name, .. } | Self::Success { test_name, .. } => test_name,
			}
		}

		pub fn get_path(&self) -> &str {
			match self {
				Self::Success { file_path, .. } | Self::Error { file_path, .. } => file_path,
			}
		}

		pub fn get_in_hash(&self) -> &str {
			match self {
				Self::Success { in_hash, .. } | Self::Error { in_hash, .. } => in_hash,
			}
		}

		pub fn get_stripped_hash(&self) -> Option<&str> {
			match self {
				Self::Success { stripped_hash, .. } => Some(stripped_hash),
				Self::Error { stripped_hash, .. } => *stripped_hash,
			}
		}

		pub fn get_audio_hash(&self) -> Option<&str> {
			match self {
				Self::Success { audio_hash, .. } => Some(audio_hash),
				_ => None,
			}
		}

		pub fn get_blocks(&self) -> Option<&[FlacBlockOutput]> {
			match self {
				Self::Success { blocks, .. } => Some(blocks),
				_ => None,
			}
		}

		pub fn get_pictures(&self) -> Option<Vec<&FlacBlockOutput>> {
			match self {
				Self::Success { blocks, .. } => {
					let mut out = Vec::new();
					for b in *blocks {
						match b {
							FlacBlockOutput::Picture { .. } => out.push(b),
							FlacBlockOutput::VorbisComment { pictures, .. } => {
								for p in *pictures {
									out.push(p)
								}
							}
							_ => {}
						}
					}

					return Some(out);
				}

				Self::Error { pictures, .. } => pictures.map(|x| x.iter().collect()),
			}
		}
	}

	/// A list of test files and their expected output
	pub const MANIFEST: &[FlacTestCase] = &[
		FlacTestCase::Error {
			test_name: "uncommon_10",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_uncommon/10 - file starting at frame header.flac"
			),
			in_hash: "d95f63e8101320f5ac7ffe249bc429a209eb0e10996a987301eaa63386a8faa1",
			check_error: &|x| matches!(x, FlacDecodeError::BadMagicBytes),
			stripped_hash: None,
			pictures: None,
		},
		FlacTestCase::Error {
			test_name: "faulty_06",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_faulty/06 - missing streaminfo metadata block.flac"
			),
			in_hash: "53aed5e7fde7a652b82ba06a8382b2612b02ebbde7b0d2016276644d17cc76cd",
			check_error: &|x| matches!(x, FlacDecodeError::BadFirstBlock),
			stripped_hash: None,
			pictures: None,
		},
		FlacTestCase::Error {
			test_name: "faulty_07",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_faulty/07 - other metadata blocks preceding streaminfo metadata block.flac"
			),
			in_hash: "6d46725991ba5da477187fde7709ea201c399d00027257c365d7301226d851ea",
			check_error: &|x| matches!(x, FlacDecodeError::BadFirstBlock),
			stripped_hash: None,
			pictures: None,
		},
		FlacTestCase::Error {
			test_name: "faulty_10",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_faulty/10 - invalid vorbis comment metadata block.flac"
			),
			in_hash: "c79b0514a61634035a5653c5493797bbd1fcc78982116e4d429630e9e462d29b",
			check_error: &|x| {
				matches!(
					x,
					FlacDecodeError::VorbisComment(VorbisCommentDecodeError::MalformedData)
				)
			},
			// This file's vorbis comment is invalid, but that shouldn't stop us from removing it.
			// As a general rule, we should NOT encounter an error when stripping invalid blocks.
			//
			// We should, however, get errors when we try to strip flac files with invalid *structure*
			// (For example, the out-of-order streaminfo test in faulty_07).
			stripped_hash: Some("4b994f82dc1699a58e2b127058b37374220ee41dc294d4887ac14f056291a1b0"),
			pictures: None,
		},
		FlacTestCase::Error {
			test_name: "faulty_11",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_faulty/11 - incorrect metadata block length.flac"
			),
			in_hash: "3732151ba8c4e66a785165aa75a444aad814c16807ddc97b793811376acacfd6",
			check_error: &|x| matches!(x, FlacDecodeError::BadMetablockType(127)),
			stripped_hash: None,
			pictures: None,
		},
		FlacTestCase::Success {
			test_name: "subset_45",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/45 - no total number of samples set.flac"
			),
			in_hash: "336a18eb7a78f7fc0ab34980348e2895bc3f82db440a2430d9f92e996f889f9a",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 907,
					max_frame_size: 8053,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 0,
					md5_signature: "c41ae3b82c35d8f5c3dab1729f948fde",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
			],
			audio_hash: "3fb3482ebc1724559bdd57f34de472458563d78a676029614e76e32b5d2b8816",
			stripped_hash: "31631ac227ebe2689bac7caa1fa964b47e71a9f1c9c583a04ea8ebd9371508d0",
		},
		FlacTestCase::Success {
			test_name: "subset_46",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/46 - no min-max framesize set.flac"
			),
			in_hash: "9dc39732ce17815832790901b768bb50cd5ff0cd21b28a123c1cabc16ed776cc",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 0,
					max_frame_size: 0,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 282866,
					md5_signature: "fd131e6ebc75251ed83f8f4c07df36a4",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
			],
			audio_hash: "a1eed422462b386a932b9eb3dff3aea3687b41eca919624fb574aadb7eb50040",
			stripped_hash: "9e57cd77f285fc31f87fa4e3a31ab8395d68d5482e174c8e0d0bba9a0c20ba27",
		},
		FlacTestCase::Success {
			test_name: "subset_47",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/47 - only STREAMINFO.flac"
			),
			in_hash: "9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
			blocks: &[FlacBlockOutput::Streaminfo {
				min_block_size: 4096,
				max_block_size: 4096,
				min_frame_size: 4747,
				max_frame_size: 7034,
				sample_rate: 48000,
				channels: 2,
				bits_per_sample: 16,
				total_samples: 232608,
				md5_signature: "bba30c5f70789910e404b7ac727c3853",
			}],
			audio_hash: "5ee1450058254087f58c91baf0f70d14bde8782cf2dc23c741272177fe0fce6e",
			stripped_hash: "9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
		},
		FlacTestCase::Success {
			test_name: "subset_48",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/48 - Extremely large SEEKTABLE.flac"
			),
			in_hash: "4417aca6b5f90971c50c28766d2f32b3acaa7f9f9667bd313336242dae8b2531",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 2445,
					max_frame_size: 7364,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 259884,
					md5_signature: "97a0574290237563fbaa788ad77d2cdf",
				},
				FlacBlockOutput::Seektable {
					hash: "21ca2184ae22fe26b690fd7cbd8d25fcde1d830ff6e5796ced4107bab219d7c0",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
			],
			audio_hash: "c2d691f2c4c986fe3cd5fd7864d9ba9ce6dd68a4ffc670447f008434b13102c2",
			stripped_hash: "abc9a0c40a29c896bc6e1cc0b374db1c8e157af716a5a3c43b7db1591a74c4e8",
		},
		FlacTestCase::Success {
			test_name: "subset_49",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/49 - Extremely large PADDING.flac",
			),
			in_hash: "7bc44fa2754536279fde4f8fb31d824f43b8d0b3f93d27d055d209682914f20e",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 1353,
					max_frame_size: 7117,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 258939,
					md5_signature: "6e78f221caaaa5d570a53f1714d84ded",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
				FlacBlockOutput::Padding { size: 16777215 },
			],
			audio_hash: "5007be7109b28b0149d1b929d2a0e93a087381bd3e68cf2a3ef78ea265ea20c3",
			stripped_hash: "a2283bbacbc4905ad3df1bf9f43a0ea7aa65cf69523d84a7dd8eb54553cc437e",
		},
		FlacTestCase::Success {
			test_name: "subset_50",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/50 - Extremely large PICTURE.flac"
			),
			in_hash: "1f04f237d74836104993a8072d4223e84a5d3bd76fbc44555c221c7e69a23594",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 5099,
					max_frame_size: 7126,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 265617,
					md5_signature: "82164e4da30ed43b47e6027cef050648",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
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
			],
			audio_hash: "9778b25c5d1f56cfcd418e550baed14f9d6a4baf29489a83ed450fbebb28de8c",
			stripped_hash: "20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c",
		},
		FlacTestCase::Success {
			test_name: "subset_51",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/51 - Extremely large VORBISCOMMENT.flac"
			),
			in_hash: "033160e8124ed287b0b5d615c94ac4139477e47d6e4059b1c19b7141566f5ef9",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 4531,
					max_frame_size: 7528,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 289972,
					md5_signature: "5ff622c88f8dd9bc201a6a541f3890d3",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Hash {
						n_comments: 39,
						hash: "01984e9ec0cfad41f27b3b4e84184966f6725ead84b7815bd0b3313549ee4229",
					},
					pictures: &[],
				},
			],
			audio_hash: "76419865d10eb22a74f020423a4e515e800f0177441676afd0418557c2d76c36",
			stripped_hash: "c0ca6c6099b5d9ec53d6bb370f339b2b1570055813a6cd3616fac2db83a2185e",
		},
		FlacTestCase::Success {
			test_name: "subset_52",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/52 - Extremely large APPLICATION.flac"
			),
			in_hash: "0e45a4f8dbef15cbebdd8dfe690d8ae60e0c6abb596db1270a9161b62a7a3f1c",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 3711,
					max_frame_size: 7056,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 317876,
					md5_signature: "eb7140266bc194527488c21ab49bc47b",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
				FlacBlockOutput::Application {
					application_id: 0x74657374,
					hash: "cfc0b8969e4ba6bd507999ba89dea2d274df69d94749d6ae3cf117a7780bba09",
				},
			],
			audio_hash: "89ad1a5c86a9ef35d33189c81c8a90285a23964a13f8325bf2c02043e8c83d63",
			stripped_hash: "cc4a0afb95ec9bcde8ee33f13951e494dc4126a9a3a668d79c80ce3c14a3acd9",
		},
		FlacTestCase::Success {
			test_name: "subset_53",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/53 - CUESHEET with very many indexes.flac"
			),
			in_hash: "513fad18578f3225fae5de1bda8f700415be6fd8aa1e7af533b5eb796ed2d461",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 2798,
					max_frame_size: 7408,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 2910025,
					md5_signature: "d11f3717d628cfe6a90a10facc478340",
				},
				FlacBlockOutput::Seektable {
					hash: "18629e1b874cb27e4364da72fb3fec2141eb0618baae4a1cee6ed09562aa00a8",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
				FlacBlockOutput::CueSheet {
					hash: "70638a241ca06881a52c0a18258ea2d8946a830137a70479c49746d2a1344bdd",
				},
			],
			audio_hash: "e993070f2080f2c598be1d61d208e9187a55ddea4be1d2ed1f8043e7c03e97a5",
			stripped_hash: "57c5b945e14c6fcd06916d6a57e5b036d67ff35757893c24ed872007aabbcf4b",
		},
		FlacTestCase::Success {
			test_name: "subset_54",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/54 - 1000x repeating VORBISCOMMENT.flac"
			),
			in_hash: "b68dc6644784fac35aa07581be8603a360d1697e07a2265d7eb24001936fd247",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 1694,
					max_frame_size: 7145,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 433151,
					md5_signature: "1d950e92b357dedbc5290a7f2210a2ef",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Hash {
						n_comments: 20000,
						hash: "433f34ae532d265835153139b1db79352a26ad0d3b03e2f1a1b88ada34abfc77",
					},
					pictures: &[],
				},
			],
			audio_hash: "4721b784058410c6263f73680079e9a71aee914c499afcf5580c121fce00e874",
			stripped_hash: "5c8b92b83c0fa17821add38263fa323d1c66cfd2ee57aca054b50bd05b9df5c2",
		},
		FlacTestCase::Success {
			test_name: "subset_55",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/55 - file 48-53 combined.flac"
			),
			in_hash: "a756b460df79b7cc492223f80cda570e4511f2024e5fa0c4d505ba51b86191f6",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 3103,
					max_frame_size: 11306,
					sample_rate: 44100,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 2646000,
					md5_signature: "2c78978cbbff11daac296fee97c3e061",
				},
				FlacBlockOutput::Seektable {
					hash: "58dfa7bac4974edf1956b068f5aa72d1fbd9301c36a3085a8a57b9db11a2dbf0",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.3 20190804",
					comments: VorbisCommentTestValue::Hash {
						n_comments: 40036,
						hash: "66cac9f9c42f48128e9fc24e1e96b46a06e885d233155556da16d9b05a23486e",
					},
					pictures: &[],
				},
				FlacBlockOutput::CueSheet {
					hash: "db11916c8f5f39648256f93f202e00ff8d73d7d96b62f749b4c77cf3ea744f90",
				},
				FlacBlockOutput::Application {
					application_id: 0x74657374,
					hash: "6088a557a1bad7bfa5ebf79a324669fbf4fa2f8e708f5487305dfc5b2ff2249a",
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
				FlacBlockOutput::Padding { size: 16777215 },
			],
			audio_hash: "f1285b77cec7fa9a0979033244489a9d06b8515b2158e9270087a65a4007084d",
			stripped_hash: "401038fce06aff5ebdc7a5f2fc01fa491cbf32d5da9ec99086e414b2da3f8449",
		},
		FlacTestCase::Success {
			test_name: "subset_56",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/56 - JPG PICTURE.flac"
			),
			in_hash: "5cebe7a3710cf8924bd2913854e9ca60b4cd53cfee5a3af0c3c73fddc1888963",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 3014,
					max_frame_size: 7219,
					sample_rate: 44100,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 220026,
					md5_signature: "5b0e898d9c2626d0c28684f5a586813f",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
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
			audio_hash: "ccfe90b0f15cd9662f7a18f40cd4c347538cf8897a08228e75351206f7804573",
			stripped_hash: "31a38d59db2010790b7abf65ec0cc03f2bbe1fed5952bc72bee4ca4d0c92e79f",
		},
		FlacTestCase::Success {
			test_name: "subset_57",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/57 - PNG PICTURE.flac"
			),
			in_hash: "c6abff7f8bb63c2821bd21dd9052c543f10ba0be878e83cb419c248f14f72697",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 463,
					max_frame_size: 6770,
					sample_rate: 44100,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 221623,
					md5_signature: "ad16957bcf8d5a3ec8caf261e43d5ff7",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
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
			audio_hash: "39bf9981613ac2f35d253c0c21b76a48abba7792c27da5dbf23e6021e2e6673f",
			stripped_hash: "3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1",
		},
		FlacTestCase::Success {
			test_name: "subset_58",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/58 - GIF PICTURE.flac"
			),
			in_hash: "7c2b1a963a665847167a7275f9924f65baeb85c21726c218f61bf3f803f301c8",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 2853,
					max_frame_size: 6683,
					sample_rate: 44100,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 219826,
					md5_signature: "7c1810602a7db96d7a48022ac4aa495c",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
				FlacBlockOutput::Picture {
					picture_type: PictureType::FrontCover,
					mime: MimeType::Gif,
					description: "",
					width: 1920,
					height: 1080,
					bit_depth: 24,
					color_count: 32,
					img_data: "e33cccc1d799eb2bb618f47be7099cf02796df5519f3f0e1cc258606cf6e8bb1",
				},
			],
			audio_hash: "30e3292e9f56cf88658eeadfdec8ad3a440690ce6d813e1b3374f60518c8e0ae",
			stripped_hash: "4cd771e27870e2a586000f5b369e0426183a521b61212302a2f5802b046910b2",
		},
		FlacTestCase::Success {
			test_name: "subset_59",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_subset/59 - AVIF PICTURE.flac"
			),
			in_hash: "7395d02bf8d9533dc554cce02dee9de98c77f8731a45f62d0a243bd0d6f9a45c",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 153,
					max_frame_size: 7041,
					sample_rate: 44100,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 221423,
					md5_signature: "d354246011ca204159c06f52cad5f634",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[],
				},
				FlacBlockOutput::Picture {
					picture_type: PictureType::FrontCover,
					mime: MimeType::Avif,
					description: "",
					width: 1920,
					height: 1080,
					bit_depth: 24,
					color_count: 0,
					img_data: "a431123040c74f75096237f20544a7fb56b4eb71ddea62efa700b0a016f5b2fc",
				},
			],
			audio_hash: "b208c73d274e65b27232bfffbfcbcf4805ee3cbc9cfbf7d2104db8f53370273b",
			stripped_hash: "d5215e16c6b978fc2c3e6809e1e78981497cb8514df297c5169f3b4a28fd875c",
		},
		FlacTestCase::Success {
			test_name: "custom_01",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_custom/01 - many images.flac"
			),
			in_hash: "8a5df37488866cd91ac16773e549ef4e3a85d9f88a0d9d345f174807bb536b96",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 5099,
					max_frame_size: 7126,
					sample_rate: 48000,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 265617,
					md5_signature: "82164e4da30ed43b47e6027cef050648",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[FlacBlockOutput::Picture {
						picture_type: PictureType::FrontCover,
						mime: MimeType::Png,
						description: "",
						width: 960,
						height: 540,
						bit_depth: 24,
						color_count: 0,
						img_data:
							"d804e5c7b9ee5af694b5e301c6cdf64508ff85997deda49d2250a06a964f10b2",
					}],
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
					mime: MimeType::Gif,
					description: "dolor",
					width: 1920,
					height: 1080,
					bit_depth: 24,
					color_count: 32,
					img_data: "e33cccc1d799eb2bb618f47be7099cf02796df5519f3f0e1cc258606cf6e8bb1",
				},
				FlacBlockOutput::Picture {
					picture_type: PictureType::BackCover,
					mime: MimeType::Avif,
					description: "est",
					width: 1920,
					height: 1080,
					bit_depth: 24,
					color_count: 0,
					img_data: "a431123040c74f75096237f20544a7fb56b4eb71ddea62efa700b0a016f5b2fc",
				},
			],
			audio_hash: "9778b25c5d1f56cfcd418e550baed14f9d6a4baf29489a83ed450fbebb28de8c",
			stripped_hash: "20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c",
		},
		FlacTestCase::Success {
			test_name: "custom_02",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_custom/02 - picture in vorbis comment.flac"
			),
			in_hash: "f6bb1a726fe6a3e25a4337d36e29fdced8ff01a46d627b7c2e1988c88f461f8c",
			blocks: &[
				FlacBlockOutput::Streaminfo {
					min_block_size: 4096,
					max_block_size: 4096,
					min_frame_size: 463,
					max_frame_size: 6770,
					sample_rate: 44100,
					channels: 2,
					bits_per_sample: 16,
					total_samples: 221623,
					md5_signature: "ad16957bcf8d5a3ec8caf261e43d5ff7",
				},
				FlacBlockOutput::VorbisComment {
					vendor: "reference libFLAC 1.3.2 20170101",
					comments: VorbisCommentTestValue::Raw { tags: &[] },
					pictures: &[FlacBlockOutput::Picture {
						picture_type: PictureType::FrontCover,
						mime: MimeType::Png,
						description: "",
						width: 960,
						height: 540,
						bit_depth: 24,
						color_count: 0,
						img_data:
							"d804e5c7b9ee5af694b5e301c6cdf64508ff85997deda49d2250a06a964f10b2",
					}],
				},
			],
			audio_hash: "39bf9981613ac2f35d253c0c21b76a48abba7792c27da5dbf23e6021e2e6673f",
			stripped_hash: "3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1",
		},
		FlacTestCase::Error {
			test_name: "custom_03",
			file_path: concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/tests/files/flac_custom/03 - faulty picture in vorbis comment.flac"
			),
			in_hash: "7177f0ae4f04a563292be286ec05967f81ab16eb0a28b70fc07a1e47da9cafd0",
			check_error: &|x| {
				matches!(
					x,
					FlacDecodeError::VorbisComment(VorbisCommentDecodeError::MalformedPicture)
				)
			},
			stripped_hash: Some("3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1"),
			pictures: None,
		},
	];

	#[test]
	fn manifest_sanity_check() {
		assert!(MANIFEST.iter().map(|x| x.get_name()).all_unique());
		assert!(MANIFEST.iter().map(|x| x.get_path()).all_unique());
	}
}
