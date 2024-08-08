use ufo_pipeline::api::{PipelineNode, PipelineNodeError, PipelineNodeState};

use crate::{
	data::{BytesSource, UFOData, UFODataStub},
	traits::UFOStaticNode,
	UFOContext,
};

pub struct FileReader {
	path: Option<UFOData>,
}

impl FileReader {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		FileReader { path: None }
	}
}

impl PipelineNode for FileReader {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineNodeError> {
		match port {
			0 => {
				self.path = match data {
					UFOData::Bytes {
						source: BytesSource::File { .. },
						..
					} => Some(data),
					x => panic!("bad data {x:?}"),
				};
			}
			_ => unreachable!("bad input port {port}"),
		}

		Ok(())
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineNodeError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineNodeError>,
	{
		if let Some(path) = self.path.take() {
			send_data(0, path)?;
			return Ok(PipelineNodeState::Done);
		} else {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}
	}
}

impl UFOStaticNode for FileReader {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("path", UFODataStub::Bytes)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Bytes)]
	}
}
