//! A flac processor that finds all images inside a flac file

use super::super::{
	blockread::{FlacBlock, FlacBlockReader, FlacBlockReaderError, FlacBlockSelector},
	blocks::FlacPictureBlock,
};

// TODO: extract picture from vorbis tags

/// Find all pictures in a flac file
pub struct FlacPictureReader {
	reader: FlacBlockReader,
}

impl FlacPictureReader {
	/// Make a new [`FlacMetaStrip`]
	pub fn new() -> Self {
		Self {
			reader: FlacBlockReader::new(FlacBlockSelector {
				pick_streaminfo: false,
				pick_padding: false,
				pick_application: false,
				pick_seektable: false,
				pick_vorbiscomment: false,
				pick_cuesheet: false,
				pick_picture: true,
				pick_audio: false,
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
		!self.reader.is_done() || self.reader.has_block()
	}

	/// Pop the next picture we read from this file, if any.
	pub fn pop_picture(&mut self) -> Option<FlacPictureBlock> {
		match self.reader.pop_block() {
			Some(FlacBlock::Picture(p)) => Some(p),
			None => None,
			_ => unreachable!(),
		}
	}
}

#[cfg(test)]
mod tests {
	use paste::paste;
	use rand::Rng;
	use sha2::{Digest, Sha256};

	use crate::flac::{
		blockread::FlacBlockReaderError,
		proc::pictures::FlacPictureReader,
		tests::{FlacBlockOutput, FlacTestCase, MANIFEST},
	};

	fn test_pictures(
		test_case: &FlacTestCase,
		fragment_size_range: Option<std::ops::Range<usize>>,
	) -> Result<(), FlacBlockReaderError> {
		let file_data = std::fs::read(test_case.get_path()).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		hasher.update(&file_data);
		assert_eq!(test_case.get_in_hash(), format!("{:x}", hasher.finalize()));

		let mut pic = FlacPictureReader::new();

		// Push file data to the reader, in parts or as a whole.
		if let Some(fragment_size_range) = fragment_size_range {
			let mut head = 0;
			while head < file_data.len() {
				let mut frag_size = rand::thread_rng().gen_range(fragment_size_range.clone());
				if head + frag_size > file_data.len() {
					frag_size = file_data.len() - head;
				}
				pic.push_data(&file_data[head..head + frag_size])?;
				head += frag_size;
			}
		} else {
			pic.push_data(&file_data)?;
		}

		pic.finish()?;

		let mut out = Vec::new();
		while let Some(p) = pic.pop_picture() {
			out.push(p);
		}

		let out_pictures = test_case.get_pictures().unwrap();

		assert_eq!(
			out.len(),
			out_pictures.len(),
			"Unexpected number of pictures"
		);

		for (got, expected) in out.iter().zip(out_pictures) {
			let (picture_type, mime, description, width, height, bit_depth, color_count, img_data) =
				match expected {
					FlacBlockOutput::Picture {
						picture_type,
						mime,
						description,
						width,
						height,
						bit_depth,
						color_count,
						img_data,
					} => (
						picture_type,
						mime,
						description,
						width,
						height,
						bit_depth,
						color_count,
						img_data,
					),
					_ => unreachable!(),
				};

			assert_eq!(*picture_type, got.picture_type);
			assert_eq!(*mime, got.mime);
			assert_eq!(*description, got.description);
			assert_eq!(*width, got.width);
			assert_eq!(*height, got.height);
			assert_eq!(*bit_depth, got.bit_depth);
			assert_eq!(*color_count, got.color_count);
			assert_eq!(img_data, {
				let mut hasher = Sha256::new();
				hasher.update(&got.img_data);
				&format!("{:x}", hasher.finalize())
			});
		}

		return Ok(());
	}

	macro_rules! gen_tests {
		( $test_name:ident ) => {
			paste! {
				#[test]
				pub fn [<strip_small_ $test_name>]() {
					let test_case = MANIFEST.iter().find(|x| x.get_name() == stringify!($test_name)).unwrap();
					match test_case {
						FlacTestCase::Error { pictures: Some(_), .. } |
						FlacTestCase::Success { .. } => {
							for _ in 0..5 {
								test_pictures(
									test_case,
									Some(1..256),
								).unwrap()
							}
						},

						FlacTestCase::Error { check_error, .. } => {
							let e = test_pictures(test_case, Some(1..256)).unwrap_err();
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
						FlacTestCase::Error { pictures: Some(_), .. } |
						FlacTestCase::Success { .. } => {
							for _ in 0..5 {
								test_pictures(
									test_case,
									Some(5_000..100_000),
								).unwrap()
							}
						},

						FlacTestCase::Error { check_error, .. } => {
							let e = test_pictures(test_case, Some(5_000..100_000)).unwrap_err();
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
