use copper_util::MimeType;
use std::{
	fmt::Debug,
	io::{Cursor, Read},
};

use crate::{
	common::picturetype::PictureType,
	flac::errors::{FlacDecodeError, FlacEncodeError},
};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A picture metablock in a flac file
pub struct FlacPictureBlock {
	/// The type of this picture
	pub picture_type: PictureType,

	/// The format of this picture
	pub mime: MimeType,

	/// The description of this picture
	pub description: String,

	/// The width of this picture, in px
	pub width: u32,

	/// The height of this picture, in px
	pub height: u32,

	/// The bit depth of this picture
	pub bit_depth: u32,

	/// The color count of this picture (if indexed)
	pub color_count: u32,

	/// The image data
	pub img_data: Vec<u8>,
}

impl Debug for FlacPictureBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FlacPicture")
			.field("type", &self.picture_type)
			.field("mime", &self.mime)
			.finish()
	}
}

impl FlacMetablockDecode for FlacPictureBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		let mut d = Cursor::new(data);

		// This is re-used whenever we need to read four bytes
		let mut block = [0u8; 4];

		d.read_exact(&mut block)
			.map_err(|_| FlacDecodeError::MalformedBlock)?;
		let picture_type = PictureType::from_idx(u32::from_be_bytes(block))?;

		// Image format
		let mime = {
			d.read_exact(&mut block)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			let mime_length = u32::from_be_bytes(block).try_into().unwrap();
			let mut mime = vec![0u8; mime_length];

			d.read_exact(&mut mime)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			String::from_utf8(mime)?.into()
		};

		// Image description
		let description = {
			d.read_exact(&mut block)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			let desc_length = u32::from_be_bytes(block).try_into().unwrap();
			let mut desc = vec![0u8; desc_length];

			d.read_exact(&mut desc)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			String::from_utf8(desc)?
		};

		// Image width
		d.read_exact(&mut block)
			.map_err(|_| FlacDecodeError::MalformedBlock)?;
		let width = u32::from_be_bytes(block);

		// Image height
		d.read_exact(&mut block)
			.map_err(|_| FlacDecodeError::MalformedBlock)?;
		let height = u32::from_be_bytes(block);

		// Image bit depth
		d.read_exact(&mut block)
			.map_err(|_| FlacDecodeError::MalformedBlock)?;
		let depth = u32::from_be_bytes(block);

		// Color count for indexed images
		d.read_exact(&mut block)
			.map_err(|_| FlacDecodeError::MalformedBlock)?;
		let color_count = u32::from_be_bytes(block);

		// Image data length
		let img_data = {
			d.read_exact(&mut block)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			let data_length = u32::from_be_bytes(block).try_into().unwrap();
			let mut img_data = vec![0u8; data_length];

			d.read_exact(&mut img_data)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			img_data
		};

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

impl FlacMetablockEncode for FlacPictureBlock {
	fn get_len(&self) -> u32 {
		(4 + (4 + self.mime.to_string().len())
			+ (4 + self.description.len())
			+ 4 + 4 + 4 + 4
			+ (4 + self.img_data.len()))
		.try_into()
		.unwrap()
	}

	fn encode(
		&self,
		is_last: bool,
		with_header: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		if with_header {
			let header = FlacMetablockHeader {
				block_type: FlacMetablockType::Picture,
				length: self.get_len(),
				is_last,
			};
			header.encode(target)?;
		}

		target.write_all(&self.picture_type.to_idx().to_be_bytes())?;

		let mime = self.mime.to_string();
		target.write_all(&u32::try_from(mime.len()).unwrap().to_be_bytes())?;
		target.write_all(self.mime.to_string().as_bytes())?;
		drop(mime);

		target.write_all(&u32::try_from(self.description.len()).unwrap().to_be_bytes())?;
		target.write_all(self.description.to_string().as_bytes())?;

		target.write_all(&self.width.to_be_bytes())?;
		target.write_all(&self.height.to_be_bytes())?;
		target.write_all(&self.bit_depth.to_be_bytes())?;
		target.write_all(&self.color_count.to_be_bytes())?;

		target.write_all(&u32::try_from(self.img_data.len()).unwrap().to_be_bytes())?;
		target.write_all(&self.img_data)?;

		return Ok(());
	}
}
