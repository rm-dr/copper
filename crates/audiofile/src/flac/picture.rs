//! Decode FLAC picture metadata blocks

use std::{
	fmt::{Debug, Display},
	io::Read,
	string::FromUtf8Error,
};

use ufo_util::mime::MimeType;

use crate::common::picturetype::{PictureType, PictureTypeError};

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
