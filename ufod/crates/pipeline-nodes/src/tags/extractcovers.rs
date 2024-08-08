use std::{io::Read, sync::Arc};
use ufo_audiofile::flac::proc::pictures::FlacPictureReader;
use ufo_pipeline::api::{PipelineNode, PipelineNodeError, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{BytesSource, UFOData, UFODataStub},
	helpers::DataSource,
	traits::UFOStaticNode,
	UFOContext,
};

pub struct ExtractCovers {
	blob_fragment_size: u64,
	data: DataSource,
	reader: FlacPictureReader,
}

impl ExtractCovers {
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			blob_fragment_size: ctx.blob_fragment_size,

			data: DataSource::Uninitialized,
			reader: FlacPictureReader::new(),
		}
	}
}

impl PipelineNode for ExtractCovers {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineNodeError> {
		match port {
			0 => match data {
				UFOData::Bytes { source, mime } => {
					if mime != MimeType::Flac {
						return Err(PipelineNodeError::UnsupportedFormat(format!(
							"cannot extract covers from `{}`",
							mime
						)));
					}

					self.data.consume(mime, source);
				}

				_ => panic!("bad input type"),
			},

			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineNodeError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineNodeError>,
	{
		// Push latest data into cover reader
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(PipelineNodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
				while let Some(d) = data.pop_front() {
					self.reader
						.push_data(&d)
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.reader
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
			}

			DataSource::File { file, .. } => {
				let mut v = Vec::new();
				let n = file
					.by_ref()
					.take(self.blob_fragment_size)
					.read_to_end(&mut v)?;
				self.reader
					.push_data(&v)
					.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;

				if n == 0 {
					self.reader
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Send the first cover we find
		// TODO: send an array of covers
		if let Some(picture) = self.reader.pop_picture() {
			send_data(
				0,
				UFOData::Bytes {
					mime: picture.mime.clone(),
					source: BytesSource::Array {
						fragment: Arc::new(picture.img_data),
						is_last: true,
					},
				},
			)?;
			return Ok(PipelineNodeState::Done);
		} else if self.reader.is_done() {
			send_data(0, UFOData::None(UFODataStub::Bytes))?;
			return Ok(PipelineNodeState::Done);
		}

		return Ok(PipelineNodeState::Pending("No pictures yet"));
	}
}

impl UFOStaticNode for ExtractCovers {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Bytes)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("cover_data", UFODataStub::Bytes)]
	}
}
