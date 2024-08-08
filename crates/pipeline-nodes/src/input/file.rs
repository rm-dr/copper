use std::{fs::File, io::Read, path::PathBuf, sync::Arc};
use ufo_metadb::data::MetaDbDataStub;
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::UFOData, errors::PipelineError, helpers::HoldSender, traits::UFOStaticNode, UFOContext,
};

/// A node that reads data from a file
pub struct FileReader {
	blob_fragment_size: usize,
	blob_channel_capacity: usize,

	path: Option<PathBuf>,
	file: Option<File>,

	is_done: bool,
	sender: Option<HoldSender>,
}

impl FileReader {
	/// Make a new [`FileReader`]
	pub fn new(ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		FileReader {
			blob_channel_capacity: ctx.blob_channel_capacity,
			blob_fragment_size: ctx.blob_fragment_size,

			path: None,
			file: None,

			is_done: false,
			sender: None,
		}
	}
}

impl PipelineNode for FileReader {
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
				self.path = match data {
					UFOData::Path(p) => Some((*p).clone()),
					x => panic!("bad data {x:?}"),
				};

				self.file = Some(File::open(self.path.as_ref().unwrap()).unwrap());
				send_data(
					0,
					UFOData::Path(Arc::new(self.path.as_ref().unwrap().clone())),
				)?;

				// Prepare sender
				let (hs, recv) = HoldSender::new(self.blob_channel_capacity);
				self.sender = Some(hs);
				send_data(
					1,
					UFOData::Blob {
						format: {
							self.path
								.as_ref()
								.unwrap()
								.extension()
								.map(|x| {
									MimeType::from_extension(x.to_str().unwrap())
										.unwrap_or(MimeType::Blob)
								})
								.unwrap_or(MimeType::Blob)
						},
						data: recv,
					},
				)?;
			}
			_ => unreachable!("bad input port {port}"),
		}

		Ok(())
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.path.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		// If we're holding a message, try to send it
		if let Some(x) = self.sender.as_mut().unwrap().send_held_message() {
			return Ok(x);
		}

		// We've already sent all segments of our file, there's nothing to do.
		if self.is_done {
			return Ok(PipelineNodeState::Done);
		}

		loop {
			// Read a segment of our file
			let mut read_buf = Vec::with_capacity(self.blob_fragment_size);
			self.file
				.as_mut()
				.unwrap()
				.take(self.blob_fragment_size.try_into().unwrap())
				.read_to_end(&mut read_buf)
				.unwrap();

			self.is_done = read_buf.is_empty();
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

impl UFOStaticNode for FileReader {
	fn inputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("path", MetaDbDataStub::Path)]
	}

	fn outputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[
			("path", MetaDbDataStub::Path),
			("data", MetaDbDataStub::Blob),
		]
	}
}
