use std::{io::Read, sync::Arc};
use ufo_audiofile::flac::metastrip::{FlacMetaStrip, FlacMetaStripSelector};
use ufo_metadb::data::MetaDbDataStub;
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};

use crate::{
	data::UFOData,
	errors::PipelineError,
	helpers::{ArcVecBuffer, HoldSender},
	traits::UFOStaticNode,
	UFOContext,
};

pub struct StripTags {
	blob_channel_capacity: usize,
	blob_fragment_size: usize,

	data: Option<async_broadcast::Receiver<Arc<Vec<u8>>>>,

	is_done: bool,
	strip: FlacMetaStrip,
	buffer: ArcVecBuffer,
	sender: Option<HoldSender>,
}

impl StripTags {
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			blob_channel_capacity: ctx.blob_channel_capacity,
			blob_fragment_size: ctx.blob_fragment_size,

			is_done: false,
			data: None,
			sender: None,
			strip: FlacMetaStrip::new(
				FlacMetaStripSelector::new()
					.keep_streaminfo(true)
					.keep_seektable(true)
					.keep_cuesheet(true),
			),
			buffer: ArcVecBuffer::new(),
		}
	}
}

impl PipelineNode for StripTags {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input<F>(
		&mut self,
		(port, data): (usize, UFOData),
		send_data: F,
	) -> Result<(), PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		match port {
			0 => {
				// Read latest data from receiver
				let (data_type, data) = match data {
					UFOData::Blob {
						format: data_type,
						data,
					} => (data_type, data),
					_ => return Err(PipelineError::UnsupportedDataType),
				};

				self.data = Some(data);

				// Prepare receiver
				let (hs, recv) = HoldSender::new(self.blob_channel_capacity);
				self.sender = Some(hs);
				send_data(
					0,
					UFOData::Blob {
						format: data_type,
						data: recv,
					},
				)?;
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, _send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.data.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		// If we're holding a message, try to send it
		if let Some(x) = self.sender.as_mut().unwrap().send_held_message() {
			return Ok(x);
		}

		// We've already sent all our data, there's nothing left to do.
		if self.is_done {
			return Ok(PipelineNodeState::Done);
		}

		// Write new data info `self.strip`
		let (changed, done) = self.buffer.recv_all(self.data.as_mut().unwrap());
		self.is_done = done;
		match (changed, done) {
			(false, false) => return Ok(PipelineNodeState::Pending("no new data")),
			(false, true) | (true, true) | (true, false) => {}
		}
		match std::io::copy(&mut self.buffer, &mut self.strip) {
			Ok(_) => {}
			Err(e) => match self.strip.take_error() {
				Some(x) => return Err(x.into()),
				None => return Err(e.into()),
			},
		};
		self.buffer.clear();

		// Read as much as we can from `self.strip`
		loop {
			// Read a segment of our file
			let mut read_buf = Vec::with_capacity(self.blob_fragment_size);

			match Read::by_ref(&mut self.strip)
				.take(self.blob_fragment_size.try_into().unwrap())
				.read_to_end(&mut read_buf)
			{
				Ok(_) => {}
				Err(e) => match self.strip.take_error() {
					Some(x) => return Err(x.into()),
					None => return Err(e.into()),
				},
			};

			if !self.is_done {
				// If we read data, send or hold the segment we read
				if let Some(x) = self
					.sender
					.as_mut()
					.unwrap()
					.send_or_store(Arc::new(read_buf))
				{
					return Ok(x);
				}
			} else {
				if self.sender.as_ref().unwrap().is_holding() {
					// We still have a message to send
					return Ok(PipelineNodeState::Pending("done; holding message"));
				} else {
					return Ok(PipelineNodeState::Done);
				}
			}
		}
	}
}

impl UFOStaticNode for StripTags {
	fn inputs() -> &'static [(&'static str, ufo_metadb::data::MetaDbDataStub)] {
		&[("data", MetaDbDataStub::Blob)]
	}

	fn outputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("out", MetaDbDataStub::Blob)]
	}
}
