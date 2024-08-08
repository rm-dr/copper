use crossbeam::channel::Receiver;
use std::{io::Seek, sync::Arc};
use ufo_audiofile::flac::flac_read_pictures;
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{errors::PipelineError, helpers::ArcVecBuffer, traits::UFOStaticNode, UFOContext};

pub struct ExtractCovers {
	data: Option<MetaDbData>,
	buffer: ArcVecBuffer,
	input_receiver: Receiver<(usize, MetaDbData)>,
}

impl ExtractCovers {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, MetaDbData)>,
	) -> Self {
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
	type ErrorType = PipelineError;

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
			return Ok(PipelineNodeState::Pending("args not ready"));
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
			(false, true) => {
				// We couldn't read a flac metadata header,
				// probably consumed a bad stream.
				// TODO: this should be an error
				send_data(0, MetaDbData::None(MetaDbDataStub::Binary))?;
				return Ok(PipelineNodeState::Done);
			}
			(false, false) => return Ok(PipelineNodeState::Pending("no new data")),
			(true, true) | (true, false) => {}
		}

		self.buffer.seek(std::io::SeekFrom::Start(0)).unwrap();
		let picture = match data_type {
			MimeType::Flac => {
				let pictures = flac_read_pictures(&mut self.buffer);
				if pictures.is_err() {
					return Ok(PipelineNodeState::Pending("malformed pictures"));
				}
				let mut pictures = pictures.unwrap();
				pictures.pop()
			}
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		if let Some(picture) = picture {
			send_data(
				0,
				MetaDbData::Binary {
					format: picture.get_mime().clone(),
					data: Arc::new(picture.take_img_data()),
				},
			)?;
		} else {
			send_data(0, MetaDbData::None(MetaDbDataStub::Binary))?;
		}

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
