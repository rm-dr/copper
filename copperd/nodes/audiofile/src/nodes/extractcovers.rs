use crate::flac::proc::pictures::FlacPictureReader;
use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::{BytesSource, PipeData},
	helpers::DataSource,
	CopperContext,
};
use copper_storaged::AttrDataStub;
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Extract covers from an audio file
pub struct ExtractCovers {
	blob_fragment_size: u64,

	/// None if disconnected, `Uninitialized` if unset
	data: Option<DataSource>,
	reader: FlacPictureReader,
}

impl ExtractCovers {
	/// Create a new [`ExtractCovers`] node
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.is_empty() {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			blob_fragment_size: ctx.blob_fragment_size,
			reader: FlacPictureReader::new(),
			data: None,
		})
	}
}

// Inputs: "data", Bytes
// Outputs: "cover_data", Bytes
impl Node<PipeData> for ExtractCovers {
	fn process_signal(&mut self, signal: NodeSignal<PipeData>) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { port } => match port.id().as_str() {
				"data" => {
					if self.data.is_some() {
						unreachable!("tried to connect an input twice")
					}
					self.data = Some(DataSource::Uninitialized)
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::DisconnectInput { port } => match port.id().as_str() {
				"data" => {
					if self.data.is_none() {
						unreachable!("tried to disconnect an input that hasn't been connected")
					}

					if matches!(self.data, Some(DataSource::Uninitialized)) {
						return Err(ProcessSignalError::RequiredInputEmpty);
					}
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::ReceiveInput { port, data } => {
				if self.data.is_none() {
					unreachable!("received input to a disconnected port")
				}

				match port.id().as_str() {
					"data" => match data {
						PipeData::Blob { source, mime } => {
							if mime != MimeType::Flac {
								return Err(ProcessSignalError::UnsupportedFormat(format!(
									"cannot read tags from `{}`",
									mime
								)));
							}

							self.data.as_mut().unwrap().consume(mime, source);
						}

						_ => return Err(ProcessSignalError::InputWithBadType),
					},

					_ => return Err(ProcessSignalError::InputPortDoesntExist),
				}
			}
		}

		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		match self.data.as_mut() {
			None => return Err(RunNodeError::RequiredInputNotConnected),

			Some(DataSource::Uninitialized) => {
				return Ok(NodeState::Pending("No data received"));
			}

			Some(DataSource::Binary { data, is_done, .. }) => {
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

			Some(DataSource::Url { data, .. }) => {
				let mut v = Vec::new();
				let n = data.take(self.blob_fragment_size).read_to_end(&mut v)?;
				self.reader
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				if n == 0 {
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
