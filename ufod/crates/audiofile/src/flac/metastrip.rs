//! Strip metadata from a FLAC file without loading the whole thing into memory.

use std::{
	collections::VecDeque,
	io::{Cursor, ErrorKind, Read, Seek, Write},
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

	// If we encounter an error, it will be stored here.
	// If error is not none, this whole struct is poisoned.
	// `Read` and `Write` will do nothing.
	error: Option<FlacError>,
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
			error: None,
		}
	}

	/// If this struct has encountered an error, get it.
	pub fn get_error(&self) -> &Option<FlacError> {
		&self.error
	}

	/// If this struct has encountered an error, take it.
	/// When this is called, the state of this struct is reset.
	pub fn take_error(&mut self) -> Option<FlacError> {
		let x = self.error.take();
		*self = Self::new(self.selector);
		return x;
	}
}

impl Write for FlacMetaStrip {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		if self.error.is_some() {
			return Err(ErrorKind::InvalidData.into());
		}

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
								let x = FlacMetablockType::parse_header(&header[..]);
								if let Err(e) = x {
									self.error = Some(e);
									return Ok(0);
								}
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
							self.error = Some(FlacError::BadMagicBytes);
							return Ok(0);
						};
						self.output_buffer.extend(&self.current_block);
						self.current_block_total_length = 4;
						self.current_block_type = FlacMetaStripBlockType::BlockHeader;
					}

					FlacMetaStripBlockType::BlockHeader => {
						assert!(self.current_block.len() == 4);
						assert!(self.current_block_length == 4);
						let x = FlacMetablockType::parse_header(&self.current_block[..]);
						if let Err(e) = x {
							self.error = Some(e);
							return Ok(0);
						}
						let (block_type, length, is_last) = x.unwrap();
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

	fn flush(&mut self) -> std::io::Result<()> {
		if self.error.is_some() {
			return Err(ErrorKind::InvalidData.into());
		}

		return Ok(());
	}
}

impl Read for FlacMetaStrip {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		if self.error.is_some() {
			return Err(ErrorKind::InvalidData.into());
		}

		let n_to_read = buf.len().min(self.output_buffer.len());

		let x = Read::by_ref(&mut self.output_buffer)
			.take(n_to_read.try_into().unwrap())
			.read(buf)?;
		//self.output_buffer.drain(0..x);
		return Ok(x);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use sha2::{Digest, Sha256};
	use std::{
		fs::File,
		path::{Path, PathBuf},
	};

	// Reference implementation:
	// metaflac --remove-all --dont-use-padding <file>

	fn strip_test_file(
		test_file_path: &Path,
		in_hash: &str,
		out_hash: &str,
		save_to: Option<&Path>,
	) {
		let mut file = File::open(test_file_path).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		std::io::copy(&mut file, &mut hasher).unwrap();
		file.seek(std::io::SeekFrom::Start(0)).unwrap();
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, in_hash);

		let mut strip = FlacMetaStrip::new(
			FlacMetaStripSelector::new()
				.keep_streaminfo(true)
				.keep_seektable(true)
				.keep_cuesheet(true),
		);

		std::io::copy(&mut file, &mut strip).unwrap();

		let mut read_buf = Vec::new();
		strip.read_to_end(&mut read_buf).unwrap();

		// Used to inspect file if test fails
		if let Some(p) = save_to {
			let mut out = File::create(p).unwrap();
			out.write_all(&read_buf).unwrap();
		}

		// Make sure output file is correct
		let mut hasher = Sha256::new();
		hasher.update(read_buf);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, out_hash);
	}

	#[test]
	fn basic_strip_45() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/45 - no total number of samples set.flac"),
			"336a18eb7a78f7fc0ab34980348e2895bc3f82db440a2430d9f92e996f889f9a",
			"31631ac227ebe2689bac7caa1fa964b47e71a9f1c9c583a04ea8ebd9371508d0",
			None,
		)
	}

	#[test]
	fn basic_strip_46() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/46 - no min-max framesize set.flac"),
			"9dc39732ce17815832790901b768bb50cd5ff0cd21b28a123c1cabc16ed776cc",
			"9e57cd77f285fc31f87fa4e3a31ab8395d68d5482e174c8e0d0bba9a0c20ba27",
			None,
		)
	}

	#[test]
	fn basic_strip_47() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/47 - only STREAMINFO.flac"),
			"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
			"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
			None,
		)
	}

	#[test]
	fn basic_strip_48() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/48 - Extremely large SEEKTABLE.flac"),
			"4417aca6b5f90971c50c28766d2f32b3acaa7f9f9667bd313336242dae8b2531",
			"c0624c78fb5c648b5ac2973eb2b840150ee6127121e1281ba07e376acb12aa06",
			None,
		)
	}

	#[test]
	fn basic_strip_49() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/49 - Extremely large PADDING.flac"),
			"7bc44fa2754536279fde4f8fb31d824f43b8d0b3f93d27d055d209682914f20e",
			"a2283bbacbc4905ad3df1bf9f43a0ea7aa65cf69523d84a7dd8eb54553cc437e",
			None,
		)
	}

	#[test]
	fn basic_strip_50() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/50 - Extremely large PICTURE.flac"),
			"1f04f237d74836104993a8072d4223e84a5d3bd76fbc44555c221c7e69a23594",
			"20df129287d94f9ae5951b296d7f65fcbed92db423ba7db4f0d765f1f0a7e18c",
			None,
		)
	}

	#[test]
	fn basic_strip_51() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/51 - Extremely large VORBISCOMMENT.flac"),
			"033160e8124ed287b0b5d615c94ac4139477e47d6e4059b1c19b7141566f5ef9",
			"c0ca6c6099b5d9ec53d6bb370f339b2b1570055813a6cd3616fac2db83a2185e",
			None,
		)
	}

	#[test]
	fn basic_strip_52() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/52 - Extremely large APPLICATION.flac"),
			"0e45a4f8dbef15cbebdd8dfe690d8ae60e0c6abb596db1270a9161b62a7a3f1c",
			"cc4a0afb95ec9bcde8ee33f13951e494dc4126a9a3a668d79c80ce3c14a3acd9",
			None,
		)
	}

	#[test]
	fn basic_strip_53() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/53 - CUESHEET with very many indexes.flac"),
			"513fad18578f3225fae5de1bda8f700415be6fd8aa1e7af533b5eb796ed2d461",
			"c8f640a848ba20659c688ae6d29f1762600db213ca97b9e369b745233774c3c4",
			None,
		)
	}
	#[test]
	fn basic_strip_54() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/54 - 1000x repeating VORBISCOMMENT.flac"),
			"b68dc6644784fac35aa07581be8603a360d1697e07a2265d7eb24001936fd247",
			"5c8b92b83c0fa17821add38263fa323d1c66cfd2ee57aca054b50bd05b9df5c2",
			None,
		)
	}

	#[test]
	fn basic_strip_55() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/55 - file 48-53 combined.flac"),
			"a756b460df79b7cc492223f80cda570e4511f2024e5fa0c4d505ba51b86191f6",
			"3fe8ca932f6285e79e258e53e22860c745b21b919ad6c842e4df9a970056978c",
			None,
		)
	}

	#[test]
	fn basic_strip_56() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/56 - JPG PICTURE.flac"),
			"5cebe7a3710cf8924bd2913854e9ca60b4cd53cfee5a3af0c3c73fddc1888963",
			"31a38d59db2010790b7abf65ec0cc03f2bbe1fed5952bc72bee4ca4d0c92e79f",
			None,
		)
	}

	#[test]
	fn basic_strip_57() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/57 - PNG PICTURE.flac"),
			"c6abff7f8bb63c2821bd21dd9052c543f10ba0be878e83cb419c248f14f72697",
			"3328201dd56289b6c81fa90ff26cb57fa9385cb0db197e89eaaa83efd79a58b1",
			None,
		)
	}

	#[test]
	fn basic_strip_58() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/58 - GIF PICTURE.flac"),
			"7c2b1a963a665847167a7275f9924f65baeb85c21726c218f61bf3f803f301c8",
			"4cd771e27870e2a586000f5b369e0426183a521b61212302a2f5802b046910b2",
			None,
		)
	}

	#[test]
	fn basic_strip_59() {
		strip_test_file(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/59 - AVIF PICTURE.flac"),
			"7395d02bf8d9533dc554cce02dee9de98c77f8731a45f62d0a243bd0d6f9a45c",
			"d5215e16c6b978fc2c3e6809e1e78981497cb8514df297c5169f3b4a28fd875c",
			None,
		)
	}
}
