use crossbeam::channel::Receiver;
use std::{
	io::{Read, Seek},
	sync::Arc,
};
use ufo_audiofile::flac::flac_read_pictures;
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
};
use ufo_util::mime::MimeType;

use crate::{helpers::ArcVecBuffer, traits::UFOStaticNode, UFOContext};

pub struct ExtractCovers {
	data: Option<MetaDbData>,
	buffer: ArcVecBuffer,
	input_receiver: Receiver<(usize, MetaDbData)>,
}

impl ExtractCovers {
	pub fn new(input_receiver: Receiver<(usize, MetaDbData)>) -> Self {
		Self {
			data: None,
			buffer: ArcVecBuffer::new(),
			input_receiver,
		}
	}
}

impl PipelineNode for ExtractCovers {
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
					_ => unreachable!("bad input port {port}"),
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

		let (data_type, data) = match self.data.as_mut().unwrap() {
			MetaDbData::Blob {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!("bad data {:#?}", self.data),
		};

		let (changed, done) = self.buffer.recv_all(data);
		match (changed, done) {
			(false, true) => unreachable!(),
			(false, false) => return Ok(PipelineNodeState::Pending),
			(true, true) | (true, false) => {}
		}

		self.buffer.seek(std::io::SeekFrom::Start(0)).unwrap();
		let (cover_data, cover_format): (Vec<u8>, MimeType) = match data_type {
			MimeType::Flac => {
				let r = flac_read_pictures(&mut self.buffer).unwrap();
				if r.is_none() {
					return Ok(PipelineNodeState::Pending);
				};
				let mut r = r.unwrap();

				let mut x = Vec::new();
				r.read_to_end(&mut x).unwrap();
				(x, r.get_mime().clone())
			}
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		send_data(
			0,
			MetaDbData::Binary {
				format: cover_format,
				data: Arc::new(cover_data),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}

impl UFOStaticNode for ExtractCovers {
	fn inputs() -> &'static [(&'static str, ufo_metadb::data::MetaDbDataStub)] {
		&[("data", MetaDbDataStub::Blob)]
	}

	fn outputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("cover_data", MetaDbDataStub::Binary)]
	}
}
