use std::{
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_audiofile::flac::metastrip::FlacMetaStrip;
use ufo_util::{data::PipelineData, mime::MimeType};

use crate::{
	errors::PipelineError,
	nodes::{PipelineNode, PipelineNodeState},
};

#[derive(Clone)]
pub struct StripTags {
	data: Option<PipelineData>,
}

impl StripTags {
	pub fn new() -> Self {
		Self { data: None }
	}
}

impl Default for StripTags {
	fn default() -> Self {
		Self::new()
	}
}

impl PipelineNode for StripTags {
	fn init<F>(
		&mut self,
		_send_data: F,
		mut input: Vec<PipelineData>,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		let (data_type, data) = match self.data.as_ref().unwrap() {
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

		return Ok(PipelineNodeState::Done);
	}
}
