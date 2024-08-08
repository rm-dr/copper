use std::{fs::File, io::Read, path::PathBuf, str::FromStr, sync::Arc};

use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_util::mime::MimeType;

use crate::{data::UFOData, UFOContext};

pub struct FileInput {
	path: Option<PathBuf>,
}

impl FileInput {
	pub fn new() -> Self {
		FileInput { path: None }
	}
}

impl PipelineNode for FileInput {
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
		self.path = match input.pop().unwrap() {
			UFOData::Text(t) => Some(PathBuf::from_str(&t).unwrap()),
			_ => panic!(),
		};
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
		let p = self.path.as_ref().unwrap();
		let mut f = File::open(p).unwrap();
		let mut data = Vec::new();
		f.read_to_end(&mut data).unwrap();

		let file_format = MimeType::from_extension(p.extension().unwrap().to_str().unwrap())
			.unwrap_or(MimeType::Blob);

		send_data(0, UFOData::Text(Arc::new(p.to_str().unwrap().to_string())))?;

		send_data(
			1,
			UFOData::Binary {
				format: file_format,
				data: Arc::new(data),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}
