use std::sync::Arc;
use ufo_audiofile::flac::proc::metastrip::FlacMetaStrip;
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	traits::UFOStaticNode,
	UFOContext,
};

pub struct StripTags {
	strip: FlacMetaStrip,
}

impl StripTags {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			strip: FlacMetaStrip::new(),
		}
	}
}

impl PipelineNode for StripTags {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => {
				// Read latest data from receiver
				let (format, fragment, is_last) = match data {
					UFOData::Blob {
						mime: format,
						fragment,
						is_last,
					} => (format, fragment, is_last),
					_ => unreachable!(),
				};

				if format != MimeType::Flac {
					return Err(PipelineError::UnsupportedDataType(format!(
						"cannot strip tags from `{}`",
						format
					)));
				}

				self.strip.push_data(&fragment)?;
				if is_last {
					self.strip.finish()?;
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
		if self.strip.has_data() {
			let mut out = Vec::new();
			self.strip.read_data(&mut out).unwrap();

			if !out.is_empty() {
				send_data(
					0,
					UFOData::Blob {
						mime: MimeType::Flac,
						fragment: Arc::new(out),
						is_last: !self.strip.has_data(),
					},
				)?;
			}
		} else if self.strip.is_done() {
			let mut out = Vec::new();
			self.strip.read_data(&mut out).unwrap();
			return Ok(PipelineNodeState::Done);
		}

		return Ok(PipelineNodeState::Pending("Reader is waiting for data"));
	}
}

impl UFOStaticNode for StripTags {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Blob)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("out", UFODataStub::Blob)]
	}
}
