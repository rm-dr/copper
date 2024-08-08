use std::sync::Arc;
use ufo_audiofile::flac::proc::pictures::FlacPictureReader;
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	traits::UFOStaticNode,
	UFOContext,
};

pub struct ExtractCovers {
	mime: Option<MimeType>,
	reader: FlacPictureReader,
}

impl ExtractCovers {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			mime: None,
			reader: FlacPictureReader::new(),
		}
	}
}

impl PipelineNode for ExtractCovers {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => {
				let (format, fragment, is_last) = match data {
					UFOData::Blob {
						mime: format,
						fragment,
						is_last,
					} => (format, fragment, is_last),
					_ => unreachable!(),
				};

				if let Some(f) = &self.mime {
					assert!(*f == format);
				} else {
					self.mime = Some(format);
				}

				self.reader.push_data(&fragment)?;
				if is_last {
					self.reader.finish()?;
				}
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.mime.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		if let Some(picture) = self.reader.pop_picture() {
			send_data(
				0,
				UFOData::Binary {
					mime: picture.mime.clone(),
					data: Arc::new(picture.img_data),
				},
			)?;
			return Ok(PipelineNodeState::Done);
		} else if self.reader.is_done() {
			send_data(0, UFOData::None(UFODataStub::Binary))?;
			return Ok(PipelineNodeState::Done);
		}

		return Ok(PipelineNodeState::Pending("No pictures yet"));
	}
}

impl UFOStaticNode for ExtractCovers {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Blob)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("cover_data", UFODataStub::Binary)]
	}
}
