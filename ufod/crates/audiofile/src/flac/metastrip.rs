//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::{
	collections::VecDeque,
	io::{Cursor, Read, Seek},
};

use super::{errors::FlacError, metablocktype::FlacMetablockType};

// TODO: tests
// TODO: detect end of file using STREAMINFO

/// Select which blocks we want to keep.
/// All values are `false` by default.
#[derive(Debug, Default, Clone, Copy)]
pub struct FlacMetaStripSelector {
	/// If true, keep `FlacMetablockType::Streaminfo` blocks.
	keep_streaminfo: bool,

	/// If true, keep `FlacMetablockType::Padding` blocks.
	keep_padding: bool,

	/// If true, keep `FlacMetablockType::Application` blocks.
	keep_application: bool,

	/// If true, keep `FlacMetablockType::SeekTable` blocks.
	keep_seektable: bool,

	/// If true, keep `FlacMetablockType::VorbisComment` blocks.
	keep_vorbiscomment: bool,

	/// If true, keep `FlacMetablockType::CueSheet` blocks.
	keep_cuesheet: bool,

	/// If true, keep `FlacMetablockType::Picture` blocks.
	keep_picture: bool,
}

impl FlacMetaStripSelector {
	/// Make a new [`FlacMetaStripSelector`]
	pub fn new() -> Self {
		Self::default()
	}

	fn select(&self, block_type: FlacMetablockType) -> bool {
		match block_type {
			FlacMetablockType::Streaminfo => self.keep_streaminfo,
			FlacMetablockType::Padding => self.keep_padding,
			FlacMetablockType::Application => self.keep_application,
			FlacMetablockType::Seektable => self.keep_seektable,
			FlacMetablockType::VorbisComment => self.keep_vorbiscomment,
			FlacMetablockType::Cuesheet => self.keep_cuesheet,
			FlacMetablockType::Picture => self.keep_picture,
		}
	}

	/// If true, keep `FlacMetablockType::StreamInfo` blocks.
	/// This should usually be `true`, since StreamInfo is mandatory.
	pub fn keep_streaminfo(mut self, keep_streaminfo: bool) -> Self {
		self.keep_streaminfo = keep_streaminfo;
		self
	}

	/// If true, keep `FlacMetablockType::Padding` blocks.
	pub fn keep_padding(mut self, keep_padding: bool) -> Self {
		self.keep_padding = keep_padding;
		self
	}

	/// If true, keep `FlacMetablockType::Application` blocks.
	pub fn keep_application(mut self, keep_application: bool) -> Self {
		self.keep_application = keep_application;
		self
	}

	/// If true, keep `FlacMetablockType::SeekTable` blocks.
	/// This should usually be `true`; The seek table makes seeking flac faster.
	pub fn keep_seektable(mut self, keep_seektable: bool) -> Self {
		self.keep_seektable = keep_seektable;
		self
	}

	/// If true, keep `FlacMetablockType::VorbisComment` blocks.
	pub fn keep_vorbiscomment(mut self, keep_vorbiscomment: bool) -> Self {
		self.keep_vorbiscomment = keep_vorbiscomment;
		self
	}

	/// If true, keep `FlacMetablockType::CueSheet` blocks.
	pub fn keep_cuesheet(mut self, keep_cuesheet: bool) -> Self {
		self.keep_cuesheet = keep_cuesheet;
		self
	}

	/// If true, keep `FlacMetablockType::Picture` blocks.
	pub fn keep_picture(mut self, keep_picture: bool) -> Self {
		self.keep_picture = keep_picture;
		self
	}
}

#[derive(Debug, Clone, Copy)]
enum FlacMetaStripBlockType {
	MagicBits,
	BlockHeader {
		is_first: bool,
	},
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
/// Use `push` to add flac data into this struct,
/// `Read` the same flac data but with the specified blocks removed.
///
/// This struct does not validate the content of the blocks it produces;
/// it only validates their structure (e.g, is length correct?).
pub struct FlacMetaStrip {
	// Which blocks should we keep?
	selector: FlacMetaStripSelector,

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

impl Read for FlacMetaStrip {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let n_to_read = buf.len().min(self.output_buffer.len());
		let x = Read::by_ref(&mut self.output_buffer)
			.take(n_to_read.try_into().unwrap())
			.read(buf)?;

		return Ok(x);
	}
}

impl FlacMetaStrip {
	/// Make a new [`FlacMetaStrip`].
	pub fn new(selector: FlacMetaStripSelector) -> Self {
		Self {
			selector,
			current_block: Vec::new(),
			current_block_total_length: 4,
			current_block_type: FlacMetaStripBlockType::MagicBits,
			done_with_meta: false,
			last_kept_block: None,
			current_block_length: 0,
			output_buffer: VecDeque::new(),
		}
	}

	/// Pass the given data through this metadata stripper.
	/// Output data is stored in an internal buffer, and should be accessed
	/// through `Read`.
	pub fn push_data(&mut self, buf: &[u8]) -> Result<usize, FlacError> {
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
							self.last_kept_block =
								Some((header, std::mem::take(&mut self.current_block)));
						}

						if self.done_with_meta {
							// We just read the last metadata block.
							// Append last_kept_block and prepare to read audio data
							if let Some((header, block)) = self.last_kept_block.take() {
								let (block_type, length, _) =
									FlacMetablockType::parse_header(&header[..])?;
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
							self.current_block_type =
								FlacMetaStripBlockType::BlockHeader { is_first: false };
						}
					}

					FlacMetaStripBlockType::MagicBits => {
						assert!(self.current_block.len() == 4);
						assert!(self.current_block_length == 4);
						if self.current_block != [0x66, 0x4C, 0x61, 0x43] {
							return Err(FlacError::BadMagicBytes);
						};
						self.output_buffer.extend(&self.current_block);
						self.current_block_total_length = 4;
						self.current_block_type =
							FlacMetaStripBlockType::BlockHeader { is_first: true };
					}

					FlacMetaStripBlockType::BlockHeader { is_first } => {
						assert!(self.current_block.len() == 4);
						assert!(self.current_block_length == 4);
						let (block_type, length, is_last) =
							FlacMetablockType::parse_header(&self.current_block[..])?;

						if is_first && block_type != FlacMetablockType::Streaminfo {
							return Err(FlacError::BadFirstBlock);
						}

						self.done_with_meta = is_last;
						self.current_block_total_length = length.try_into().unwrap();
						self.current_block_type = FlacMetaStripBlockType::MetaBlock {
							header: self.current_block[..].try_into().unwrap(),
							keep_this_block: self.selector.select(block_type),
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
				let read = if really_read {
					usize::try_from(std::io::copy(
						&mut buf.by_ref().take(current_block_left.try_into().unwrap()),
						&mut self.current_block,
					)?)
					.unwrap()
				} else {
					buf.seek(std::io::SeekFrom::Current(
						current_block_left
							.min(buf.get_ref().len() - written)
							.try_into()
							.unwrap(),
					))?;
					current_block_left.min(buf.get_ref().len() - written)
				};
				self.current_block_length += read;

				if read == 0 {
					return Ok(written);
				} else {
					written += read;
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use paste::paste;
	use rand::Rng;
	use sha2::{Digest, Sha256};
	use std::path::{Path, PathBuf};

	fn strip_test_whole(
		selector: FlacMetaStripSelector,
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
	) -> Result<(), Option<FlacError>> {
		let file_data = std::fs::read(test_file_path).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, in_hash);

		let mut strip = FlacMetaStrip::new(selector);

		strip.push_data(&file_data)?;
		let mut read_buf = Vec::new();
		strip.read_to_end(&mut read_buf).unwrap();

		// Make sure output file is correct
		let mut hasher = Sha256::new();
		hasher.update(read_buf);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, out_hash, "Stripped FLAC hash doesn't match");

		return Ok(());
	}

	fn strip_test_parts(
		selector: FlacMetaStripSelector,
		fragment_size_range: std::ops::Range<usize>,
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
	) -> Result<(), Option<FlacError>> {
		let file_data = std::fs::read(test_file_path).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, in_hash);

		let mut strip = FlacMetaStrip::new(selector);

		let mut head = 0;
		let mut read_buf = Vec::new();

		loop {
			let mut frag_size = rand::thread_rng().gen_range(fragment_size_range.clone());
			if head + frag_size > file_data.len() {
				frag_size = file_data.len() - head;
			}
			strip.push_data(&file_data[head..head + frag_size])?;
			head += frag_size;

			if head >= file_data.len() {
				break;
			}

			let frag_size = rand::thread_rng().gen_range(fragment_size_range.clone());
			let mut out_buf = vec![0; frag_size];
			let n = strip.read(&mut out_buf).unwrap();
			out_buf.truncate(n);
			read_buf.append(&mut out_buf);
		}

		let mut out_buf = Vec::new();
		strip.read_to_end(&mut out_buf).unwrap();
		read_buf.append(&mut out_buf);

		// Make sure output file is correct
		let mut hasher = Sha256::new();
		hasher.update(read_buf);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, out_hash, "Stripped FLAC hash doesn't match");

		return Ok(());
	}

	/*
		Strip all tests

		Reference implementation:
		`metaflac --remove-all --dont-use-padding <file>`
	*/
	fn strip_all(
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
	) -> Result<(), Option<FlacError>> {
		strip_test_whole(
			FlacMetaStripSelector::new().keep_streaminfo(true),
			test_file_path,
			in_hash,
			out_hash,
		)
	}

	/*
		Strip most tests

		Reference implementation:
		```
		metaflac
			--remove
			--block-type=PADDING,APPLICATION,VORBIS_COMMENT,PICTURE
			--dont-use-padding
			<file>
		```
	*/
	fn strip_most(
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
	) -> Result<(), Option<FlacError>> {
		strip_test_whole(
			FlacMetaStripSelector::new()
				.keep_streaminfo(true)
				.keep_seektable(true)
				.keep_cuesheet(true),
			test_file_path,
			in_hash,
			out_hash,
		)
	}

	/// Strip a file, reading and writing in small fragments.
	fn strip_small(
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
	) -> Result<(), Option<FlacError>> {
		for _ in 0..5 {
			strip_test_parts(
				FlacMetaStripSelector::new()
					.keep_streaminfo(true)
					.keep_seektable(true)
					.keep_cuesheet(true),
				0..256,
				test_file_path,
				in_hash,
				out_hash,
			)?
		}

		return Ok(());
	}

	/// Strip a file, reading and writing in large fragments.
	fn strip_large(
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
	) -> Result<(), Option<FlacError>> {
		for _ in 0..5 {
			strip_test_parts(
				FlacMetaStripSelector::new()
					.keep_streaminfo(true)
					.keep_seektable(true)
					.keep_cuesheet(true),
				1_000_000..5_000_000,
				test_file_path,
				in_hash,
				out_hash,
			)?
		}

		return Ok(());
	}

	// Helper macro, generates tests
	macro_rules! test_success {
		(
			// The name of this test
			$file_name:ident,

			// The path to the test file
			$file_path:expr,

			// SHA-256 hash of unmodified source file
			$in_hash:literal,

			// SHA-256 hash of the original file with all tags stripped
			$all_strip_hash:literal,

			// SHA-256 hash of the original file with most tags stripped
			$most_strip_hash:literal
		) => {
			paste! {
				#[test]
				pub fn [<strip_all_ $file_name>]() {
					strip_all(
						$file_path,
						$in_hash,
						$all_strip_hash,
					)
					.unwrap()
				}

				#[test]
				pub fn [<strip_most_ $file_name>]() {
					strip_most(
						$file_path,
						$in_hash,
						$most_strip_hash,
					)
					.unwrap()
				}

				#[test]
				pub fn [<strip_small_ $file_name>]() {
					strip_small(
						$file_path,
						$in_hash,
						$most_strip_hash,
					)
					.unwrap()
				}

				#[test]
				pub fn [<strip_large_ $file_name>]() {
					strip_large(
						$file_path,
						$in_hash,
						$most_strip_hash,
					)
					.unwrap()
				}
			}
		};
	}

	test_success!(
		custom_01,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_custom/01 - many images.flac"),
		"58ee39efe51e37f51b4dedeee8b28bed88ac1d4a70ba0e3a326ef7e94f0ebf1b",
		"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c",
		"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c"
	);

	// This file has an invalid vorbis comment block, but that doesn't matter.
	// We strip the block, and the resulting file is valid.
	// TODO: should this succeed? Maybe we should check blocks as we strip them?
	//
	// The hash 4b994f82dc1699a58e2b127058b37374220ee41dc294d4887ac14f056291a1b0
	// was generated by `metaflac` after manually fixing the single-byte error.
	test_success!(
		faulty_10,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_faulty/10 - invalid vorbis comment metadata block.flac"),
		"c79b0514a61634035a5653c5493797bbd1fcc78982116e4d429630e9e462d29b",
		"4b994f82dc1699a58e2b127058b37374220ee41dc294d4887ac14f056291a1b0",
		"4b994f82dc1699a58e2b127058b37374220ee41dc294d4887ac14f056291a1b0"
	);

	test_success!(
		subset_45,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/45 - no total number of samples set.flac"),
		"336a18eb7a78f7fc0ab34980348e2895bc3f82db440a2430d9f92e996f889f9a",
		"31631ac227ebe2689bac7caa1fa964b47e71a9f1c9c583a04ea8ebd9371508d0",
		"31631ac227ebe2689bac7caa1fa964b47e71a9f1c9c583a04ea8ebd9371508d0"
	);

	test_success!(
		subset_46,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/46 - no min-max framesize set.flac"),
		"9dc39732ce17815832790901b768bb50cd5ff0cd21b28a123c1cabc16ed776cc",
		"9e57cd77f285fc31f87fa4e3a31ab8395d68d5482e174c8e0d0bba9a0c20ba27",
		"9e57cd77f285fc31f87fa4e3a31ab8395d68d5482e174c8e0d0bba9a0c20ba27"
	);

	test_success!(
		subset_47,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/47 - only STREAMINFO.flac"),
		"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
		"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
		"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85"
	);

	test_success!(
		subset_48,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/48 - Extremely large SEEKTABLE.flac"),
		"4417aca6b5f90971c50c28766d2f32b3acaa7f9f9667bd313336242dae8b2531",
		"c0624c78fb5c648b5ac2973eb2b840150ee6127121e1281ba07e376acb12aa06",
		"abc9a0c40a29c896bc6e1cc0b374db1c8e157af716a5a3c43b7db1591a74c4e8"
	);

	test_success!(
		subset_49,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/49 - Extremely large PADDING.flac"),
		"7bc44fa2754536279fde4f8fb31d824f43b8d0b3f93d27d055d209682914f20e",
		"a2283bbacbc4905ad3df1bf9f43a0ea7aa65cf69523d84a7dd8eb54553cc437e",
		"a2283bbacbc4905ad3df1bf9f43a0ea7aa65cf69523d84a7dd8eb54553cc437e"
	);

	test_success!(
		subset_50,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/50 - Extremely large PICTURE.flac"),
		"1f04f237d74836104993a8072d4223e84a5d3bd76fbc44555c221c7e69a23594",
		"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c",
		"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c"
	);

	test_success!(
		subset_51,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/51 - Extremely large VORBISCOMMENT.flac"),
		"033160e8124ed287b0b5d615c94ac4139477e47d6e4059b1c19b7141566f5ef9",
		"c0ca6c6099b5d9ec53d6bb370f339b2b1570055813a6cd3616fac2db83a2185e",
		"c0ca6c6099b5d9ec53d6bb370f339b2b1570055813a6cd3616fac2db83a2185e"
	);

	test_success!(
		subset_52,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/52 - Extremely large APPLICATION.flac"),
		"0e45a4f8dbef15cbebdd8dfe690d8ae60e0c6abb596db1270a9161b62a7a3f1c",
		"cc4a0afb95ec9bcde8ee33f13951e494dc4126a9a3a668d79c80ce3c14a3acd9",
		"cc4a0afb95ec9bcde8ee33f13951e494dc4126a9a3a668d79c80ce3c14a3acd9"
	);

	test_success!(
		subset_53,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/53 - CUESHEET with very many indexes.flac"),
		"513fad18578f3225fae5de1bda8f700415be6fd8aa1e7af533b5eb796ed2d461",
		"c8f640a848ba20659c688ae6d29f1762600db213ca97b9e369b745233774c3c4",
		"57c5b945e14c6fcd06916d6a57e5b036d67ff35757893c24ed872007aabbcf4b"
	);

	test_success!(
		subset_54,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/54 - 1000x repeating VORBISCOMMENT.flac"),
		"b68dc6644784fac35aa07581be8603a360d1697e07a2265d7eb24001936fd247",
		"5c8b92b83c0fa17821add38263fa323d1c66cfd2ee57aca054b50bd05b9df5c2",
		"5c8b92b83c0fa17821add38263fa323d1c66cfd2ee57aca054b50bd05b9df5c2"
	);

	test_success!(
		subset_55,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/55 - file 48-53 combined.flac"),
		"a756b460df79b7cc492223f80cda570e4511f2024e5fa0c4d505ba51b86191f6",
		"3fe8ca932f6285e79e258e53e22860c745b21b919ad6c842e4df9a970056978c",
		"401038fce06aff5ebdc7a5f2fc01fa491cbf32d5da9ec99086e414b2da3f8449"
	);

	test_success!(
		subset_56,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/56 - JPG PICTURE.flac"),
		"5cebe7a3710cf8924bd2913854e9ca60b4cd53cfee5a3af0c3c73fddc1888963",
		"31a38d59db2010790b7abf65ec0cc03f2bbe1fed5952bc72bee4ca4d0c92e79f",
		"31a38d59db2010790b7abf65ec0cc03f2bbe1fed5952bc72bee4ca4d0c92e79f"
	);

	test_success!(
		subset_57,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/57 - PNG PICTURE.flac"),
		"c6abff7f8bb63c2821bd21dd9052c543f10ba0be878e83cb419c248f14f72697",
		"3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1",
		"3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1"
	);

	test_success!(
		subset_58,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/58 - GIF PICTURE.flac"),
		"7c2b1a963a665847167a7275f9924f65baeb85c21726c218f61bf3f803f301c8",
		"4cd771e27870e2a586000f5b369e0426183a521b61212302a2f5802b046910b2",
		"4cd771e27870e2a586000f5b369e0426183a521b61212302a2f5802b046910b2"
	);

	test_success!(
		subset_59,
		&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("tests/files/flac_subset/59 - AVIF PICTURE.flac"),
		"7395d02bf8d9533dc554cce02dee9de98c77f8731a45f62d0a243bd0d6f9a45c",
		"d5215e16c6b978fc2c3e6809e1e78981497cb8514df297c5169f3b4a28fd875c",
		"d5215e16c6b978fc2c3e6809e1e78981497cb8514df297c5169f3b4a28fd875c"
	);

	/*
	// TODO: count audio data samples.
	// This test should pass.
	// Also, add a "truncated" test file

	#[test]
	fn strip_all_faulty_05() {
		let res = strip_all(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/05 - wrong total number of samples.flac"),
			"92f98457511f2f9413445f2fb3e236ae92eab86c38e0082dfbe9dd96d01ba92c",
			"unreachable-will-error",
			None,
		);

		// This file is missing a STREAMINFO block
		match res {
			Err(Some(FlacError::TODO)) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}
	*/

	#[test]
	fn strip_all_faulty_06() {
		let res = strip_all(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/06 - missing streaminfo metadata block.flac"),
			"53aed5e7fde7a652b82ba06a8382b2612b02ebbde7b0d2016276644d17cc76cd",
			"unreachable-will-error",
		);

		// This file is missing a STREAMINFO block
		match res {
			Err(Some(FlacError::BadFirstBlock)) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}

	#[test]
	fn strip_all_faulty_07() {
		let res = strip_all(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/07 - other metadata blocks preceding streaminfo metadata block.flac"),
			"6d46725991ba5da477187fde7709ea201c399d00027257c365d7301226d851ea",
			"unreachable-will-error",
		);

		// This file has a STREAMINFO block, but it isn't first
		match res {
			Err(Some(FlacError::BadFirstBlock)) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}

	#[test]
	fn strip_all_faulty_11() {
		let res = strip_all(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/11 - incorrect metadata block length.flac"),
			"3732151ba8c4e66a785165aa75a444aad814c16807ddc97b793811376acacfd6",
			"unreachable-will-error",
		);

		// This file has a bad block length, which results in us reading garbage data
		match res {
			Err(Some(FlacError::BadMetablockType(127))) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}

	/*
		"Strip most" tests


		Reference implementation:
		```
		metaflac
			--remove
			--block-type=PADDING,APPLICATION,VORBIS_COMMENT,PICTURE
			--dont-use-padding
			<file>
		```
	*/

	#[test]
	fn strip_most_faulty_06() {
		let res = strip_most(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/06 - missing streaminfo metadata block.flac"),
			"53aed5e7fde7a652b82ba06a8382b2612b02ebbde7b0d2016276644d17cc76cd",
			"unreachable-will-error",
		);

		// This file is missing a STREAMINFO block
		match res {
			Err(Some(FlacError::BadFirstBlock)) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}

	#[test]
	fn strip_most_faulty_07() {
		let res = strip_most(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/07 - other metadata blocks preceding streaminfo metadata block.flac"),
			"6d46725991ba5da477187fde7709ea201c399d00027257c365d7301226d851ea",
			"unreachable-will-error",
		);

		// This file has a bad block length, which results in us reading garbage data
		match res {
			Err(Some(FlacError::BadFirstBlock)) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}

	#[test]
	fn strip_most_faulty_11() {
		let res = strip_most(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_faulty/11 - incorrect metadata block length.flac"),
			"3732151ba8c4e66a785165aa75a444aad814c16807ddc97b793811376acacfd6",
			"unreachable-will-error",
		);

		// This file has a bad block length, which results in us reading garbage data
		match res {
			Err(Some(FlacError::BadMetablockType(127))) => {}
			e => panic!("Unexpected result {e:?}"),
		}
	}
}
