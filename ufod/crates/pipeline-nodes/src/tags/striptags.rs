use std::{io::Read, sync::Arc};
use ufo_audiofile::flac::proc::metastrip::FlacMetaStrip;
use ufo_pipeline::api::{PipelineNode, PipelineNodeError, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{BytesSource, UFOData, UFODataStub},
	helpers::DataSource,
	traits::UFOStaticNode,
	UFOContext,
};

pub struct StripTags {
	blob_fragment_size: u64,
	data: DataSource,
	strip: FlacMetaStrip,
}

impl StripTags {
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			blob_fragment_size: ctx.blob_fragment_size,

			strip: FlacMetaStrip::new(),
			data: DataSource::Uninitialized,
		}
	}
}

impl PipelineNode for StripTags {
	type NodeContext = UFOContext;
	type DataType = UFOData;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineNodeError> {
		match port {
			0 => match data {
				UFOData::Bytes { source, mime } => {
					if mime != MimeType::Flac {
						return Err(PipelineNodeError::UnsupportedFormat(format!(
							"cannot strip tags from `{}`",
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
		// Push latest data into metadata stripper
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(PipelineNodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
				while let Some(d) = data.pop_front() {
					self.strip
						.push_data(&d)
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.strip
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
			}

			DataSource::File { file, .. } => {
				let mut v = Vec::new();

				// Read in parts so we don't never have to load the whole
				// file into memory
				let n = file
					.by_ref()
					.take(self.blob_fragment_size)
					.read_to_end(&mut v)?;

				self.strip
					.push_data(&v)
					.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;

				if n == 0 {
					self.strip
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Read and send stripped data
		if self.strip.has_data() {
			let mut out = Vec::new();
			self.strip.read_data(&mut out).unwrap();

			if !out.is_empty() {
				send_data(
					0,
					UFOData::Bytes {
						mime: MimeType::Flac,
						source: BytesSource::Array {
							fragment: Arc::new(out),
							is_last: !self.strip.has_data(),
						},
					},
				)?;
			}
		}

		if self.strip.is_done() {
			let mut out = Vec::new();
			self.strip.read_data(&mut out).unwrap();
			return Ok(PipelineNodeState::Done);
		} else {
			return Ok(PipelineNodeState::Pending("Waiting for more data"));
		}
	}
}

impl UFOStaticNode for StripTags {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Bytes)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("out", UFODataStub::Bytes)]
	}
}
