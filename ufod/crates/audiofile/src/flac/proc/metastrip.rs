//! A flac processor that strips metadata blocks from flac files

use std::io::Write;

use super::super::{
	blockread::{FlacBlock, FlacBlockReader, FlacBlockReaderError, FlacBlockSelector},
	errors::FlacEncodeError,
};

/// Removes all metadata from a flac file
pub struct FlacMetaStrip {
	reader: FlacBlockReader,

	/// The last block that `reader` produced.
	///
	/// We need this to detect the last metadata block
	/// that `reader` produces.
	last_block: Option<FlacBlock>,

	/// Set to `false` on the first call to `self.write_data`.
	/// Used to write fLaC magic bytes.
	first_write: bool,
}

impl FlacMetaStrip {
	/// Make a new [`FlacMetaStrip`]
	pub fn new() -> Self {
		Self {
			first_write: true,
			last_block: None,
			reader: FlacBlockReader::new(FlacBlockSelector {
				pick_streaminfo: true,
				pick_padding: false,
				pick_application: false,
				pick_seektable: true,
				pick_vorbiscomment: false,
				pick_cuesheet: true,
				pick_picture: false,
				pick_audio: true,
			}),
		}
	}

	/// Push some data to this flac processor
	pub fn push_data(&mut self, buf: &[u8]) -> Result<(), FlacBlockReaderError> {
		self.reader.push_data(buf)
	}

	/// Call after sending the entire flac file to this reader
	pub fn finish(&mut self) -> Result<(), FlacBlockReaderError> {
		self.reader.finish()
	}

	/// If true, we have received all the data we need
	pub fn is_done(&mut self) -> bool {
		self.reader.is_done()
	}

	/// If false, this reader has sent all its data.
	///
	/// Note that `read_data` may write zero bytes if this method returns `true`.
	/// If `has_data` is false, we don't AND WON'T have data. If we're waiting
	/// for data, this is `true`.
	pub fn has_data(&self) -> bool {
		self.last_block.is_some() || !self.reader.is_done() || self.reader.has_block()
	}

	/// Write available data from this struct into `target`
	pub fn read_data(&mut self, target: &mut impl Write) -> Result<(), FlacEncodeError> {
		if self.first_write {
			target.write_all(&[0x66, 0x4C, 0x61, 0x43])?;
			self.first_write = false;
		}

		while let Some(block) = self.reader.pop_block() {
			if let Some(last_block) = self.last_block.take() {
				last_block.encode(
					// The last metadata block is the only one followed by an audio frame
					!matches!(last_block, FlacBlock::AudioFrame(_))
						&& matches!(block, FlacBlock::AudioFrame(_)),
					target,
				)?;
			}
			self.last_block = Some(block);
		}

		// We don't need to store audioframes in our last_block buffer,
		// since they do not have an `is_last` flag.
		if matches!(self.last_block, Some(FlacBlock::AudioFrame(_))) {
			let x = self.last_block.take().unwrap();
			x.encode(false, target)?;
		}

		return Ok(());
	}
}

#[cfg(test)]
mod tests {
	use paste::paste;
	use rand::Rng;
	use sha2::{Digest, Sha256};

	use crate::flac::{
		blockread::FlacBlockReaderError,
		proc::metastrip::FlacMetaStrip,
		tests::{FlacTestCase, MANIFEST},
	};

	fn test_strip(
		test_case: &FlacTestCase,
		fragment_size_range: Option<std::ops::Range<usize>>,
	) -> Result<(), FlacBlockReaderError> {
		let file_data = std::fs::read(test_case.get_path()).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		assert_eq!(test_case.get_in_hash(), format!("{:x}", hasher.finalize()));

		let mut strip = FlacMetaStrip::new();

		// Push file data to the reader, in parts or as a whole.
		if let Some(fragment_size_range) = fragment_size_range {
			let mut head = 0;
			while head < file_data.len() {
				let mut frag_size = rand::thread_rng().gen_range(fragment_size_range.clone());
				if head + frag_size > file_data.len() {
					frag_size = file_data.len() - head;
				}
				strip.push_data(&file_data[head..head + frag_size])?;
				head += frag_size;
			}
		} else {
			strip.push_data(&file_data)?;
		}

		strip.finish()?;

		let mut out_data = Vec::new();
		strip.read_data(&mut out_data).unwrap();
		assert!(!strip.has_data());

		let mut hasher = Sha256::new();
		hasher.update(out_data);
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(
			result,
			test_case.get_stripped_hash().unwrap(),
			"Stripped FLAC hash doesn't match"
		);

		return Ok(());
	}

	macro_rules! gen_tests {
		( $test_name:ident ) => {
			paste! {
				#[test]
				pub fn [<strip_small_ $test_name>]() {
					let test_case = MANIFEST.iter().find(|x| x.get_name() == stringify!($test_name)).unwrap();
					match test_case {
						FlacTestCase::Error { stripped_hash: Some(_), .. } |
						FlacTestCase::Success { .. } => {
							for _ in 0..5 {
								test_strip(
									test_case,
									Some(1..256),
								).unwrap()
							}
						},

						FlacTestCase::Error { check_error, .. } => {
							let e = test_strip(test_case, Some(1..256)).unwrap_err();
							match e {
								FlacBlockReaderError::DecodeError(e) => assert!(check_error(&e), "Unexpected error {e:?}"),
								_ => panic!("Unexpected error {e:?}")
							}
						}
					}
				}

				#[test]
				pub fn [<strip_large_ $test_name>]() {
					let test_case = MANIFEST.iter().find(|x| x.get_name() == stringify!($test_name)).unwrap();
					match test_case {
						FlacTestCase::Error { stripped_hash: Some(_), .. } |
						FlacTestCase::Success { .. } => {
							for _ in 0..5 {
								test_strip(
									test_case,
									Some(5_000..100_000),
								).unwrap()
							}
						},

						FlacTestCase::Error { check_error, .. } => {
							let e = test_strip(test_case, Some(5_000..100_000)).unwrap_err();
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
	gen_tests!(custom_01);
}
