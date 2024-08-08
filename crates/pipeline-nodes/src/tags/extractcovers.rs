use std::{
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_audiofile::flac::flac_read_pictures;
use ufo_pipeline::{
	data::PipelineData,
	errors::PipelineError,
	node::{PipelineNode, PipelineNodeState},
};
use ufo_util::mime::MimeType;

use crate::UFOContext;

#[derive(Clone)]
pub struct ExtractCovers {
	data: Option<PipelineData>,
}

impl ExtractCovers {
	pub fn new() -> Self {
		Self { data: None }
	}
}

impl PipelineNode for ExtractCovers {
	type RunContext = UFOContext;

	fn init<F>(
		&mut self,
		_ctx: Arc<UFOContext>,
		mut input: Vec<PipelineData>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: Arc<UFOContext>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let (data_type, data) = match self.data.as_ref().unwrap() {
			PipelineData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!("bad data {:#?}", self.data),
		};

		let data_read = Cursor::new(&**data);
		let (cover_data, cover_format): (Vec<u8>, MimeType) = match data_type {
			MimeType::Flac => {
				let mut r = flac_read_pictures(data_read).unwrap().unwrap();
				let mut x = Vec::new();
				r.read_to_end(&mut x).unwrap();
				(x, r.get_mime().clone())
			}
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		send_data(
			0,
			PipelineData::Binary {
				format: cover_format,
				data: Arc::new(cover_data),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}
