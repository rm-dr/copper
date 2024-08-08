use std::io::Cursor;
use ufo_audiofile::flac::flac_read_pictures;
use ufo_util::{data::PipelineData, mime::MimeType};

use crate::{errors::PipelineError, PipelineNode};

#[derive(Clone)]
pub struct ExtractCovers {}

impl ExtractCovers {
	pub fn new() -> Self {
		Self {}
	}
}

impl PipelineNode for ExtractCovers {
	fn run<F>(&self, _send_data: F, input: Vec<PipelineData>) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let data = input.first().unwrap();

		let (data_type, data) = match data {
			PipelineData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!("bad data {data:#?}"),
		};

		let data_read = Cursor::new(&**data);
		let _tagger = match data_type {
			MimeType::Flac => flac_read_pictures(data_read).unwrap(),
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		//println!("{:?}", tagger);

		return Ok(());
	}
}
