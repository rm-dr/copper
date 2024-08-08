use crossbeam::channel::Receiver;
use std::{
	io::{Read, Seek},
	sync::Arc,
};
use ufo_audiofile::flac::metastrip::FlacMetaStrip;
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_util::mime::MimeType;

use crate::{
	helpers::{ArcVecBuffer, HoldSender},
	traits::UFOStaticNode,
	UFOContext,
};

pub struct StripTags {
	blob_channel_capacity: usize,
	_blob_fragment_size: usize,

	data: Option<MetaDbData>,

	buffer: ArcVecBuffer,
	input_receiver: Receiver<(usize, MetaDbData)>,
	sender: Option<HoldSender>,
}

impl StripTags {
	pub fn new(
		ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, MetaDbData)>,
	) -> Self {
		Self {
			blob_channel_capacity: ctx.blob_channel_capacity,
			_blob_fragment_size: ctx.blob_fragment_size,

			data: None,
			sender: None,
			buffer: ArcVecBuffer::new(),
			input_receiver,
		}
	}
}

impl PipelineNode for StripTags {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;

	fn take_input<F>(&mut self, _send_data: F) -> Result<(), PipelineError>
	where
		F: Fn(usize, MetaDbData) -> Result<(), PipelineError>,
	{
		loop {
			match self.input_receiver.try_recv() {
				Err(crossbeam::channel::TryRecvError::Disconnected)
				| Err(crossbeam::channel::TryRecvError::Empty) => {
					break Ok(());
				}
				Ok((port, data)) => match port {
					0 => {
						self.data = Some(data);
					}
					_ => unreachable!(),
				},
			}
		}
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.data.is_none() {
			return Ok(PipelineNodeState::Pending);
		}

		// If we're holding a message, try to send it
		if let Some(sender) = &mut self.sender {
			if let Some(x) = sender.send_held_message() {
				return Ok(x);
			}
		}

		// Read latest data from receiver
		let (data_type, data) = match self.data.as_mut().unwrap() {
			MetaDbData::Blob {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		let (changed, done) = self.buffer.recv_all(data);
		match (changed, done) {
			(false, true) => unreachable!(),
			(false, false) => return Ok(PipelineNodeState::Pending),
			(true, true) | (true, false) => {}
		}

		// Try to strip metadata
		// TODO: buffer inside strip, write() into it
		self.buffer.seek(std::io::SeekFrom::Start(0)).unwrap();
		let stripped = match data_type {
			MimeType::Flac => {
				let x = FlacMetaStrip::new(&mut self.buffer);
				if x.is_err() {
					return Ok(PipelineNodeState::Pending);
				}
				let mut x = x.unwrap();
				let mut v = Vec::new();

				let r = x.read_to_end(&mut v);
				if r.is_err() {
					return Ok(PipelineNodeState::Pending);
				}
				v
			}
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		// If we haven't made a sender yet, make one
		if self.sender.is_none() {
			let (hs, recv) = HoldSender::new(self.blob_channel_capacity);
			self.sender = Some(hs);

			send_data(
				0,
				MetaDbData::Blob {
					format: data_type.clone(),
					data: recv,
				},
			)?;
		}

		if let Some(x) = self
			.sender
			.as_mut()
			.unwrap()
			.send_or_store(Arc::new(stripped))
		{
			return Ok(x);
		} else if self.sender.as_ref().unwrap().is_holding() {
			// We still have a message to send
			return Ok(PipelineNodeState::Pending);
		} else {
			return Ok(PipelineNodeState::Done);
		};
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
