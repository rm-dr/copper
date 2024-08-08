use std::{
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_audiofile::flac::metastrip::FlacMetaStrip;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_util::mime::MimeType;

use crate::{data::UFOData, UFOContext};

#[derive(Clone)]
pub struct StripTags {
	data: Option<UFOData>,
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
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn init<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		mut input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.data = Some(input.pop().unwrap());
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: Arc<Self::NodeContext>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let (data_type, data) = match self.data.as_ref().unwrap() {
			UFOData::Binary {
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
			UFOData::Binary {
				format: data_type.clone(),
				data: Arc::new(stripped),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}
