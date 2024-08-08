use std::{
	io::{Read, Write},
	sync::Arc,
};
use ufo_audiofile::flac::metastrip::{FlacMetaStrip, FlacMetaStripSelector};
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	traits::UFOStaticNode,
	UFOContext,
};

pub struct StripTags {
	blob_fragment_size: usize,

	is_done: bool,
	strip: FlacMetaStrip,
}

impl StripTags {
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			blob_fragment_size: ctx.blob_fragment_size,

			is_done: false,
			strip: FlacMetaStrip::new(
				FlacMetaStripSelector::new()
					.keep_streaminfo(true)
					.keep_seektable(true)
					.keep_cuesheet(true),
			),
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

				assert!(!self.is_done);
				self.is_done = is_last;
				self.strip.write_all(&fragment)?;
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		// Read a segment of our file
		let mut read_buf = Vec::with_capacity(self.blob_fragment_size);
		match Read::by_ref(&mut self.strip)
			.take(self.blob_fragment_size.try_into().unwrap())
			.read_to_end(&mut read_buf)
		{
			Ok(n) => n,
			Err(e) => match self.strip.take_error() {
				Some(x) => return Err(x.into()),
				None => return Err(e.into()),
			},
		};
		assert!(read_buf.len() <= self.blob_fragment_size);
		let empty = read_buf.is_empty();

		// The last fragment we send is always empty and done.
		// This prevents us from accidentally skipping the last
		// part of the file.

		if !empty || self.is_done {
			send_data(
				0,
				UFOData::Blob {
					mime: MimeType::Flac,
					fragment: Arc::new(read_buf),
					is_last: self.is_done && empty,
				},
			)?;
		}

		if self.is_done && empty {
			return Ok(PipelineNodeState::Done);
		} else {
			return Ok(PipelineNodeState::Pending("waiting for data"));
		}
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
