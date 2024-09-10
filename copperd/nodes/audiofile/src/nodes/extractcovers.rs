use crate::flac::proc::pictures::FlacPictureReader;
use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::{BytesSource, PipeData},
	helpers::{BytesSourceArrayReader, ConnectedInput, OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_storaged::AttrDataStub;
use copper_util::MimeType;
use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Extract covers from an audio file
pub struct ExtractCovers {
	data: ConnectedInput<OpenBytesSourceReader>,
	reader: FlacPictureReader,
}

impl ExtractCovers {
	/// Create a new [`ExtractCovers`] node
	pub fn new(
		_ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.is_empty() {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			reader: FlacPictureReader::new(),
			data: ConnectedInput::NotConnected,
		})
	}
}

// Inputs: "data", Bytes
// Outputs: "cover_data", Bytes
impl Node<PipeData, CopperContext> for ExtractCovers {
	fn process_signal(
		&mut self,
		ctx: &CopperContext,
		signal: NodeSignal<PipeData>,
	) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { port } => match port.id().as_str() {
				"data" => self.data.connect(),
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::DisconnectInput { port } => match port.id().as_str() {
				"data" => {
					if !self.data.is_connected() {
						unreachable!("disconnected an input that hasn't been connected")
					}
					if !self.data.is_set() {
						return Err(ProcessSignalError::RequiredInputEmpty);
					}
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::ReceiveInput { port, data } => match port.id().as_str() {
				"data" => match data {
					PipeData::Blob { source, mime } => {
						if mime != MimeType::Flac {
							return Err(ProcessSignalError::UnsupportedFormat(format!(
								"cannot read tags from `{}`",
								mime
							)));
						}

						match source {
							BytesSource::Array { .. } => {
								self.data.set(OpenBytesSourceReader::Array(
									BytesSourceArrayReader::new(Some(mime), source).unwrap(),
								));
							}

							BytesSource::S3 { key } => {
								self.data
									.set(OpenBytesSourceReader::S3(block_on(S3Reader::new(
										ctx.objectstore_client.clone(),
										&ctx.objectstore_bucket,
										key,
									))))
							}
						}
					}

					_ => return Err(ProcessSignalError::InputWithBadType),
				},

				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},
		}

		return Ok(());
	}

	fn run(
		&mut self,
		ctx: &CopperContext,
		send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		if !self.data.is_connected() {
			return Err(RunNodeError::RequiredInputNotConnected);
		}

		if !self.data.is_set() {
			return Ok(NodeState::Pending("input not ready"));
		}

		match self.data.value_mut().unwrap() {
			OpenBytesSourceReader::Array(BytesSourceArrayReader { data, is_done, .. }) => {
				while let Some(d) = data.pop_front() {
					self.reader
						.push_data(&d)
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.reader
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}

			OpenBytesSourceReader::S3(r) => {
				let mut v = Vec::new();
				r.take(ctx.blob_fragment_size).read(&mut v).unwrap();
				self.reader
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				if r.is_done() {
					self.reader
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Send the first cover we find
		if let Some(picture) = self.reader.pop_picture() {
			send_data(
				PortName::new("cover_data"),
				PipeData::Blob {
					mime: picture.mime.clone(),
					source: BytesSource::Array {
						fragment: Arc::new(picture.img_data),
						is_last: true,
					},
				},
			)?;
			return Ok(NodeState::Done);
		} else if self.reader.is_done() {
			send_data(
				PortName::new("cover_data"),
				PipeData::None {
					data_type: AttrDataStub::Blob,
				},
			)?;
			return Ok(NodeState::Done);
		}

		return Ok(NodeState::Pending("No pictures yet"));
	}
}
