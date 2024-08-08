use std::{fs::File, io::Read, path::PathBuf, sync::Arc};
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	traits::UFOStaticNode,
	UFOContext,
};

/// A node that reads data from a file
pub struct FileReader {
	blob_fragment_size: usize,

	path: Option<PathBuf>,
	sent_path: bool,
	file: Option<File>,
}

impl FileReader {
	/// Make a new [`FileReader`]
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		FileReader {
			blob_fragment_size: ctx.blob_fragment_size,

			path: None,
			sent_path: false,
			file: None,
		}
	}
}

impl PipelineNode for FileReader {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => {
				self.path = match data {
					UFOData::Path(p) => Some(p.clone()),
					x => panic!("bad data {x:?}"),
				};

				self.file = Some(File::open(self.path.as_ref().unwrap()).unwrap());
			}
			_ => unreachable!("bad input port {port}"),
		}

		Ok(())
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.path.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}
		if !self.sent_path {
			self.sent_path = true;
			send_data(0, UFOData::Path(self.path.as_ref().unwrap().clone()))?;
		}

		// Read a segment of our file
		let mut read_buf = Vec::with_capacity(self.blob_fragment_size);
		let n = self
			.file
			.as_mut()
			.unwrap()
			.take(self.blob_fragment_size.try_into().unwrap())
			.read_to_end(&mut read_buf)
			.unwrap();
		let is_last = n < self.blob_fragment_size;

		send_data(
			1,
			UFOData::Blob {
				format: {
					self.path
						.as_ref()
						.unwrap()
						.extension()
						.map(|x| {
							MimeType::from_extension(x.to_str().unwrap()).unwrap_or(MimeType::Blob)
						})
						.unwrap_or(MimeType::Blob)
				},
				fragment: Arc::new(read_buf),
				is_last,
			},
		)?;

		if is_last {
			return Ok(PipelineNodeState::Done);
		} else {
			return Ok(PipelineNodeState::Pending("more to read"));
		}
	}
}

impl UFOStaticNode for FileReader {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("path", UFODataStub::Path)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("path", UFODataStub::Path), ("data", UFODataStub::Blob)]
	}
}
