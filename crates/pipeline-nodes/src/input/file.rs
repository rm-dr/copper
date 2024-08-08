use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_storage::data::{StorageData, StorageDataStub};
use ufo_util::mime::MimeType;

use crate::{helpers::UFOStaticNode, UFOContext};

/// A node that reads data from a file
pub struct FileReader {
	path: Option<PathBuf>,
}

impl FileReader {
	/// Make a new [`FileReader`]
	pub fn new() -> Self {
		FileReader { path: None }
	}
}

impl PipelineNode for FileReader {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		mut input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 1);
		self.path = match input.pop().unwrap() {
			StorageData::Path(p) => Some((*p).clone()),
			_ => panic!(),
		};
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
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

		send_data(0, StorageData::Path(Arc::new(p.clone())))?;

		send_data(
			1,
			StorageData::Binary {
				format: file_format,
				data: Arc::new(data),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}

impl UFOStaticNode for FileReader {
	fn inputs() -> &'static [(&'static str, StorageDataStub)] {
		&[("path", StorageDataStub::Path)]
	}

	fn outputs() -> &'static [(&'static str, StorageDataStub)] {
		&[
			("path", StorageDataStub::Path),
			("data", StorageDataStub::Binary),
		]
	}
}
