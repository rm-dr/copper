//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use copper_util::mime::MimeType;
use pipelined_node_base::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PipelinePortID,
		ProcessSignalError, RunNodeError,
	},
	data::{BytesSource, CopperData},
	helpers::DataSource,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Strip all metadata from an audio file
pub struct StripTags {
	blob_fragment_size: u64,

	/// None if disconnected, `Uninitialized` if unset
	data: Option<DataSource>,
	strip: FlacMetaStrip,
}

impl StripTags {
	/// Create a new [`StripTags`] node
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.is_empty() {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			blob_fragment_size: ctx.blob_fragment_size,
			strip: FlacMetaStrip::new(),
			data: None,
		})
	}
}

// Input: "data" - Blob
// Output: "out" - Blob
impl Node<CopperData> for StripTags {
	fn process_signal(&mut self, signal: NodeSignal<CopperData>) -> Result<(), ProcessSignalError> {
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
						CopperData::Bytes { source, mime } => {
							if mime != MimeType::Flac {
								return Err(ProcessSignalError::UnsupportedFormat(format!(
									"cannot strip tags from `{}`",
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
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		// Push latest data into metadata stripper
		match self.data.as_mut() {
			None => return Err(RunNodeError::RequiredInputNotConnected),

			Some(DataSource::Uninitialized) => {
				return Ok(NodeState::Pending("input not ready"));
			}

			Some(DataSource::Binary { data, is_done, .. }) => {
				while let Some(d) = data.pop_front() {
					self.strip
						.push_data(&d)
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.strip
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}

			Some(DataSource::Url { data, .. }) => {
				let mut v = Vec::new();

				// Read in parts so we don't never have to load the whole
				// file into memory
				let n = data.take(self.blob_fragment_size).read_to_end(&mut v)?;

				self.strip
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				if n == 0 {
					self.strip
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Read and send stripped data
		if self.strip.has_data() {
			let mut out = Vec::new();
			self.strip.read_data(&mut out).unwrap();

			if !out.is_empty() {
				send_data(
					PipelinePortID::new("out"),
					CopperData::Bytes {
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
			return Ok(NodeState::Done);
		} else {
			return Ok(NodeState::Pending("Waiting for more data"));
		}
	}
}
