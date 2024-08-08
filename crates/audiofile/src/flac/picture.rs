use std::{fmt::Display, io::Read, string::FromUtf8Error};

use crate::common::{
	mime::MimeType,
	picturetype::{PictureType, PictureTypeError},
};

#[derive(Debug)]
pub enum FlacPictureError {
	IoError(std::io::Error),
	FailedStringDecode(FromUtf8Error),
	BadPictureType(PictureTypeError),
}

impl Display for FlacPictureError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "io error while reading flac picture"),
			Self::BadPictureType(_) => write!(f, "flac has invalid picture type"),
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
pub struct FlacPicture {
	picture_type: PictureType,
	mime: MimeType,
	description: String,
	width: u32,
	height: u32,
	depth: u32,
	color_count: u32,
	img_data: Vec<u8>,
}

impl FlacPicture {
	pub fn decode<R>(mut read: R) -> Result<Self, FlacPictureError>
	where
		R: Read,
	{
		// This is re-used whenever we need to read four bytes
		let mut block = [0u8; 4];

		read.read_exact(&mut block)?;
		let picture_type = PictureType::from_idx(u32::from_be_bytes(block))?;

		let mime = {
			read.read_exact(&mut block)?;
			let mime_length = u32::from_be_bytes(block);
			let mut mime = vec![0u8; mime_length.try_into().unwrap()];
			read.read_exact(&mut mime)?;
			String::from_utf8(mime)?
		}
		.into();

		let description = {
			read.read_exact(&mut block)?;
			let desc_length = u32::from_be_bytes(block);
			let mut desc = vec![0u8; desc_length.try_into().unwrap()];
			read.read_exact(&mut desc)?;
			String::from_utf8(desc)?
		};
		read.read_exact(&mut block)?;
		let width = u32::from_be_bytes(block);

		read.read_exact(&mut block)?;
		let height = u32::from_be_bytes(block);

		read.read_exact(&mut block)?;
		let depth = u32::from_be_bytes(block);

		read.read_exact(&mut block)?;
		let color_count = u32::from_be_bytes(block);

		read.read_exact(&mut block)?;
		let data_length = u32::from_be_bytes(block);
		let mut img_data = vec![0u8; data_length.try_into().unwrap()];
		read.read_exact(&mut img_data)?;

		Ok(Self {
			picture_type,
			mime,
			description,
			width,
			height,
			depth,
			color_count,
			img_data,
		})
	}
}
