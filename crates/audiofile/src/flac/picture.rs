use std::io::Read;

use crate::common::{mime::MimeType, picturetype::PictureType};

use super::errors::FlacError;

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
	pub fn decode<R>(mut read: R) -> Result<Self, FlacError>
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
