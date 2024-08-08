use std::{
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_audiofile::flac::metastrip::FlacMetaStrip;
use ufo_util::{data::PipelineData, mime::MimeType};

use crate::{errors::PipelineError, PipelineNode};

#[derive(Clone)]
pub struct StripTags {}

impl StripTags {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for StripTags {
	fn default() -> Self {
		Self::new()
	}
}

impl PipelineNode for StripTags {
	fn run<F>(&self, send_data: F, input: Vec<PipelineData>) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let data = input.first().unwrap();

		let (data_type, data) = match data {
			PipelineData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		// TODO: stream data
		let data_read = Cursor::new(&**data);
		let stripped = match data_type {
			MimeType::Flac => {
				let mut x = FlacMetaStrip::new(data_read).unwrap();
				let mut v = Vec::new();
				x.read_to_end(&mut v).unwrap();
				v
			}
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		send_data(
			0,
			PipelineData::Binary {
				format: data_type.clone(),
				data: Arc::new(stripped),
			},
		)?;

		return Ok(());
	}
}
