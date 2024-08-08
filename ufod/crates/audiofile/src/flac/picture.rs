//! Decode FLAC picture metadata blocks

use std::{
	fmt::{Debug, Display},
	io::{Read, Seek, SeekFrom},
	string::FromUtf8Error,
};

use ufo_util::mime::MimeType;

use crate::common::picturetype::{PictureType, PictureTypeError};

use super::{errors::FlacError, metablocktype::FlacMetablockType};

#[allow(missing_docs)]
#[derive(Debug)]
pub enum FlacPictureError {
	/// We encountered an i/o error while reading a block
	IoError(std::io::Error),

	/// We tried to decode a string, but found invalid UTF-8
	FailedStringDecode(FromUtf8Error),

	/// We tried to decode a picture block with an out-of-spec picture type
	BadPictureType(PictureTypeError),

	/// The picture block we're reading isn't valid
	MalformedBlock,
}

impl Display for FlacPictureError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading flac picture"),
			Self::BadPictureType(_) => write!(f, "flac picture block has invalid type"),
			Self::MalformedBlock => write!(f, "flac picture block is malformed"),
			Self::FailedStringDecode(_) => {
				write!(f, "string decode error while reading flac picture")
			}
		}
	}
}

impl std::error::Error for FlacPictureError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::IoError(x) => Some(x),
			Self::FailedStringDecode(x) => Some(x),
			_ => None,
		}
	}
}

impl From<PictureTypeError> for FlacPictureError {
	fn from(value: PictureTypeError) -> Self {
		Self::BadPictureType(value)
	}
}

impl From<std::io::Error> for FlacPictureError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<FromUtf8Error> for FlacPictureError {
	fn from(value: FromUtf8Error) -> Self {
		Self::FailedStringDecode(value)
	}
}

// TODO: enforce flac constraints and write

/// A picture metadata block in a FLAC file.
pub struct FlacPicture {
	picture_type: PictureType,
	mime: MimeType,
	description: String,
	width: u32,
	height: u32,
	bit_depth: u32,
	color_count: u32,
	img_data: Vec<u8>,
}

impl Debug for FlacPicture {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FlacPicture")
			.field("mime", &self.mime)
			.finish()
	}
}

impl FlacPicture {
	/// Try to decode a picture block from the given reader.
	///
	/// This does NOT read the picture's data. Instead, [`FlacPicture`]
	/// stores a reader produces this data. Use [`Read`] methods on
	/// [`FlacPicture`] to get this data.
	pub fn decode<R>(mut read: R) -> Result<Self, FlacPictureError>
	where
		R: Read,
	{
		// This is re-used whenever we need to read four bytes
		let mut block = [0u8; 4];
		if read.read(&mut block)? != 4 {
			return Err(FlacPictureError::MalformedBlock);
		}

		let picture_type = PictureType::from_idx(u32::from_be_bytes(block))?;

		// Image format
		let mime = {
			if read.read(&mut block)? != 4 {
				return Err(FlacPictureError::MalformedBlock);
			}
			let mime_length = u32::from_be_bytes(block).try_into().unwrap();
			let mut mime = vec![0u8; mime_length];
			if read.read(&mut mime)? != mime_length {
				return Err(FlacPictureError::MalformedBlock);
			}
			String::from_utf8(mime)?.into()
		};

		// Image description
		let description = {
			if read.read(&mut block)? != 4 {
				return Err(FlacPictureError::MalformedBlock);
			}
			let desc_length = u32::from_be_bytes(block).try_into().unwrap();
			let mut desc = vec![0u8; desc_length];
			if read.read(&mut desc)? != desc_length {
				return Err(FlacPictureError::MalformedBlock);
			}
			String::from_utf8(desc)?
		};

		// Image width
		if read.read(&mut block)? != 4 {
			return Err(FlacPictureError::MalformedBlock);
		}
		let width = u32::from_be_bytes(block);

		// Image height
		if read.read(&mut block)? != 4 {
			return Err(FlacPictureError::MalformedBlock);
		}
		let height = u32::from_be_bytes(block);

		// Image bit depth
		if read.read(&mut block)? != 4 {
			return Err(FlacPictureError::MalformedBlock);
		}
		let depth = u32::from_be_bytes(block);

		// Color count for indexed images
		if read.read(&mut block)? != 4 {
			return Err(FlacPictureError::MalformedBlock);
		}
		let color_count = u32::from_be_bytes(block);

		// Image data length
		if read.read(&mut block)? != 4 {
			return Err(FlacPictureError::MalformedBlock);
		}
		let data_length = u32::from_be_bytes(block).try_into().unwrap();
		let mut img_data = vec![0u8; data_length];
		if read.read(&mut img_data)? != data_length {
			return Err(FlacPictureError::MalformedBlock);
		}

		Ok(Self {
			picture_type,
			mime,
			description,
			width,
			height,
			bit_depth: depth,
			color_count,
			img_data,
		})
	}
}

impl FlacPicture {
	/// Get this picture's IDv3 type
	pub fn get_type(&self) -> &PictureType {
		&self.picture_type
	}

	/// Get the mime type of this image's data
	pub fn get_mime(&self) -> &MimeType {
		&self.mime
	}

	/// Get this image's description
	pub fn get_description(&self) -> &String {
		&self.description
	}

	/// Get this image's dimensions.
	/// Returns (width, height) in pixels.
	pub fn get_dimensions(&self) -> (u32, u32) {
		(self.width, self.height)
	}

	/// Get the bit depth of this image.
	pub fn get_bit_depth(&self) -> u32 {
		self.bit_depth
	}

	/// Get the number of colors in this image.
	/// 0 if this image is in a non-indexed format.
	pub fn get_color_count(&self) -> u32 {
		self.color_count
	}

	/// Get a reference to this picture's image data
	pub fn get_img_data(&self) -> &Vec<u8> {
		&self.img_data
	}

	/// Take this picture's image data
	pub fn take_img_data(self) -> Vec<u8> {
		self.img_data
	}
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
		Invalid FLAC
	*/

	/*
		Valid FLAC with no image
	*/

	#[test]
	fn picture_subset_45() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/45 - no total number of samples set.flac"),
			"336a18eb7a78f7fc0ab34980348e2895bc3f82db440a2430d9f92e996f889f9a",
			&[],
		)
	}

	#[test]
	fn picture_subset_46() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/46 - no min-max framesize set.flac"),
			"9dc39732ce17815832790901b768bb50cd5ff0cd21b28a123c1cabc16ed776cc",
			&[],
		)
	}

	#[test]
	fn picture_subset_47() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/47 - only STREAMINFO.flac"),
			"9a62c79f634849e74cb2183f9e3a9bd284f51e2591c553008d3e6449967eef85",
			&[],
		)
	}

	#[test]
	fn picture_subset_48() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/48 - Extremely large SEEKTABLE.flac"),
			"4417aca6b5f90971c50c28766d2f32b3acaa7f9f9667bd313336242dae8b2531",
			&[],
		)
	}

	#[test]
	fn picture_subset_49() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/49 - Extremely large PADDING.flac"),
			"7bc44fa2754536279fde4f8fb31d824f43b8d0b3f93d27d055d209682914f20e",
			&[],
		)
	}

	#[test]
	fn picture_subset_50() {
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
	fn picture_subset_51() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/51 - Extremely large VORBISCOMMENT.flac"),
			"033160e8124ed287b0b5d615c94ac4139477e47d6e4059b1c19b7141566f5ef9",
			&[],
		)
	}

	#[test]
	fn picture_subset_52() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/52 - Extremely large APPLICATION.flac"),
			"0e45a4f8dbef15cbebdd8dfe690d8ae60e0c6abb596db1270a9161b62a7a3f1c",
			&[],
		)
	}

	#[test]
	fn picture_subset_53() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/53 - CUESHEET with very many indexes.flac"),
			"513fad18578f3225fae5de1bda8f700415be6fd8aa1e7af533b5eb796ed2d461",
			&[],
		)
	}

	#[test]
	fn picture_subset_54() {
		fetch_images(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR"))
				.join("tests/files/flac_subset/54 - 1000x repeating VORBISCOMMENT.flac"),
			"b68dc6644784fac35aa07581be8603a360d1697e07a2265d7eb24001936fd247",
			&[],
		)
	}

	#[test]
	fn picture_subset_55() {
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
	fn picture_subset_56() {
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
	fn picture_subset_57() {
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
	fn picture_subset_58() {
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
	fn picture_subset_59() {
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
	fn picture_custom_01() {
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
