//! Parse FLAC metadata.

use std::io::{Read, Seek, SeekFrom};

use self::{errors::FlacError, metablocktype::FlacMetablockType, picture::FlacPicture};
use crate::common::vorbiscomment::VorbisComment;

pub mod errors;
pub mod metablocktype;
pub mod metastrip;
pub mod picture;
pub mod streaminfo;

/// Try to extract a vorbis comment block from the given reader.
/// `read` should provide a complete FLAC file.
pub fn flac_read_tags<R>(mut read: R) -> Result<Option<VorbisComment>, FlacError>
where
	R: Read + Seek,
{
	let mut block = [0u8; 4];
	read.read_exact(&mut block)?;
	if block != [0x66, 0x4C, 0x61, 0x43] {
		return Err(FlacError::BadMagicBytes);
	};

	// TODO: what if we have multiple vorbis blocks?
	loop {
		let (block_type, length, is_last) = FlacMetablockType::parse_header(&mut read)?;

		match block_type {
			FlacMetablockType::VorbisComment => {
				return Ok(Some(VorbisComment::decode(read.take(length.into()))?));
			}
			_ => {
				if is_last {
					break;
				} else {
					// Skip without seek:
					// io::copy(&mut file.by_ref().take(27), &mut io::sink());
					read.seek(SeekFrom::Current(length.into()))?;
					continue;
				}
			}
		};
	}

	return Ok(None);
}

/// Try to extract flac pictures from the given reader.
/// `read` should provide a complete FLAC file.
pub fn flac_read_pictures<R>(mut read: R) -> Result<Vec<FlacPicture>, FlacError>
where
	R: Read + Seek,
{
	let mut pictures = Vec::new();
	let mut block = [0u8; 4];
	read.read_exact(&mut block)?;
	if block != [0x66, 0x4C, 0x61, 0x43] {
		return Err(FlacError::BadMagicBytes);
	};

	// How about pictures in vorbis blocks?
	loop {
		let (block_type, length, is_last) = FlacMetablockType::parse_header(&mut read)?;

		match block_type {
			FlacMetablockType::Picture => {
				pictures.push(FlacPicture::decode(read.by_ref().take(length.into()))?);
			}
			_ => {
				read.seek(SeekFrom::Current(length.into()))?;
			}
		};

		if is_last {
			break;
		}
	}

	return Ok(pictures);
}

#[cfg(test)]
mod tests {
	use crate::common::picturetype::PictureType;

	use super::*;

	use sha2::{Digest, Sha256};
	use std::{
		fs::File,
		path::{Path, PathBuf},
	};
	use ufo_util::mime::MimeType;

	struct PictureData {
		picture_type: PictureType,
		mime: MimeType,
		description: &'static str,
		width: u32,
		height: u32,
		img_hash: &'static str,
	}

	fn fetch_images(test_file_path: &Path, in_hash: &str, out_images: &[PictureData]) {
		let mut file = File::open(test_file_path).unwrap();

		// Make sure input file is correct
		let mut hasher = Sha256::new();
		std::io::copy(&mut file, &mut hasher).unwrap();
		file.seek(std::io::SeekFrom::Start(0)).unwrap();
		let result = format!("{:x}", hasher.finalize());
		assert_eq!(result, in_hash);

		let pictures = flac_read_pictures(&mut file).unwrap();
		assert_eq!(pictures.len(), out_images.len());

		// Make sure output is correct
		for (p, d) in pictures.into_iter().zip(out_images) {
			assert_eq!(*p.get_type(), d.picture_type, "Picture type didn't match");
			assert_eq!(*p.get_mime(), d.mime, "Mime type didn't match");
			assert_eq!(
				*p.get_description(),
				d.description,
				"Description didn't match"
			);
			assert_eq!(p.get_dimensions().0, d.width, "Image width didn't match");
			assert_eq!(p.get_dimensions().1, d.height, "Image height didn't match");

			let mut hasher = Sha256::new();
			hasher.update(p.get_img_data());
			let result = format!("{:x}", hasher.finalize());
			assert_eq!(result, d.img_hash);
		}
	}

	/*
		Valid FLAC with no image
	*/

	#[test]
	fn image_extract_45() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/45 - no total number of samples set.flac"),
			"336a18eb7a78f7fc0ab34980348e2895bc3f82db440a2430d9f92e996f889f9a",
			&[],
		)
	}

	#[test]
	fn image_extract_46() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/46 - no min-max framesize set.flac"),
			"9dc39732ce17815832790901b768bb50cd5ff0cd21b28a123c1cabc16ed776cc",
			&[],
		)
	}

	#[test]
	fn image_extract_47() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/47 - only STREAMINFO.flac"),
			"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
			&[],
		)
	}

	#[test]
	fn image_extract_48() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/48 - Extremely large SEEKTABLE.flac"),
			"4417aca6b5f90971c50c28766d2f32b3acaa7f9f9667bd313336242dae8b2531",
			&[],
		)
	}

	#[test]
	fn image_extract_49() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/49 - Extremely large PADDING.flac"),
			"7bc44fa2754536279fde4f8fb31d824f43b8d0b3f93d27d055d209682914f20e",
			&[],
		)
	}

	#[test]
	fn image_extract_50() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/50 - Extremely large PICTURE.flac"),
			"1f04f237d74836104993a8072d4223e84a5d3bd76fbc44555c221c7e69a23594",
			&[PictureData {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 3200,
				height: 2252,
				img_hash: "b78c3a48fde4ebbe8e4090e544caeb8f81ed10020d57cc50b3265f9b338d8563",
			}],
		)
	}

	#[test]
	fn image_extract_51() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/51 - Extremely large VORBISCOMMENT.flac"),
			"033160e8124ed287b0b5d615c94ac4139477e47d6e4059b1c19b7141566f5ef9",
			&[],
		)
	}

	#[test]
	fn image_extract_52() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/52 - Extremely large APPLICATION.flac"),
			"0e45a4f8dbef15cbebdd8dfe690d8ae60e0c6abb596db1270a9161b62a7a3f1c",
			&[],
		)
	}

	#[test]
	fn image_extract_53() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/53 - CUESHEET with very many indexes.flac"),
			"513fad18578f3225fae5de1bda8f700415be6fd8aa1e7af533b5eb796ed2d461",
			&[],
		)
	}

	#[test]
	fn image_extract_54() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/54 - 1000x repeating VORBISCOMMENT.flac"),
			"b68dc6644784fac35aa07581be8603a360d1697e07a2265d7eb24001936fd247",
			&[],
		)
	}

	#[test]
	fn image_extract_55() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/55 - file 48-53 combined.flac"),
			"a756b460df79b7cc492223f80cda570e4511f2024e5fa0c4d505ba51b86191f6",
			&[PictureData {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 3200,
				height: 2252,
				img_hash: "b78c3a48fde4ebbe8e4090e544caeb8f81ed10020d57cc50b3265f9b338d8563",
			}],
		)
	}

	#[test]
	fn image_extract_56() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/56 - JPG PICTURE.flac"),
			"5cebe7a3710cf8924bd2913854e9ca60b4cd53cfee5a3af0c3c73fddc1888963",
			&[PictureData {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Jpg,
				description: "",
				width: 1920,
				height: 1080,
				img_hash: "7a3ed658f80f433eee3914fff451ea0312807de0af709e37cc6a4f3f6e8a47c6",
			}],
		)
	}

	#[test]
	fn image_extract_57() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/57 - PNG PICTURE.flac"),
			"c6abff7f8bb63c2821bd21dd9052c543f10ba0be878e83cb419c248f14f72697",
			&[PictureData {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Png,
				description: "",
				width: 960,
				height: 540,
				img_hash: "d804e5c7b9ee5af694b5e301c6cdf64508ff85997deda49d2250a06a964f10b2",
			}],
		)
	}

	#[test]
	fn image_extract_58() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/58 - GIF PICTURE.flac"),
			"7c2b1a963a665847167a7275f9924f65baeb85c21726c218f61bf3f803f301c8",
			&[PictureData {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Unknown("image/gif".into()),
				description: "",
				width: 1920,
				height: 1080,
				img_hash: "e33cccc1d799eb2bb618f47be7099cf02796df5519f3f0e1cc258606cf6e8bb1",
			}],
		)
	}

	#[test]
	fn image_extract_59() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/59 - AVIF PICTURE.flac"),
			"7395d02bf8d9533dc554cce02dee9de98c77f8731a45f62d0a243bd0d6f9a45c",
			&[PictureData {
				picture_type: PictureType::FrontCover,
				mime: MimeType::Unknown("image/avif".into()),
				description: "",
				width: 1920,
				height: 1080,
				img_hash: "a431123040c74f75096237f20544a7fb56b4eb71ddea62efa700b0a016f5b2fc",
			}],
		)
	}

	#[test]
	fn image_extract_c01() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_custom/01 - many images.flac"),
			"58ee39efe51e37f51b4dedeee8b28bed88ac1d4a70ba0e3a326ef7e94f0ebf1b",
			&[
				PictureData {
					picture_type: PictureType::FrontCover,
					mime: MimeType::Jpg,
					description: "",
					width: 3200,
					height: 2252,
					img_hash: "b78c3a48fde4ebbe8e4090e544caeb8f81ed10020d57cc50b3265f9b338d8563",
				},
				PictureData {
					picture_type: PictureType::ABrightColoredFish,
					mime: MimeType::Jpg,
					description: "lorem",
					width: 1920,
					height: 1080,
					img_hash: "7a3ed658f80f433eee3914fff451ea0312807de0af709e37cc6a4f3f6e8a47c6",
				},
				PictureData {
					picture_type: PictureType::OtherFileIcon,
					mime: MimeType::Png,
					description: "ipsum",
					width: 960,
					height: 540,
					img_hash: "d804e5c7b9ee5af694b5e301c6cdf64508ff85997deda49d2250a06a964f10b2",
				},
				PictureData {
					picture_type: PictureType::Lyricist,
					mime: MimeType::Unknown("image/gif".into()),
					description: "dolor",
					width: 1920,
					height: 1080,
					img_hash: "e33cccc1d799eb2bb618f47be7099cf02796df5519f3f0e1cc258606cf6e8bb1",
				},
				PictureData {
					picture_type: PictureType::BackCover,
					mime: MimeType::Unknown("image/avif".into()),
					description: "est",
					width: 1920,
					height: 1080,
					img_hash: "a431123040c74f75096237f20544a7fb56b4eb71ddea62efa700b0a016f5b2fc",
				},
			],
		)
	}
}
