//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::{BytesSource, PipeData},
	helpers::{BytesSourceArrayReader, ConnectedInput, OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_util::MimeType;
use futures::executor::block_on;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Strip all metadata from an audio file
pub struct StripTags {
	data: ConnectedInput<OpenBytesSourceReader>,
	strip: FlacMetaStrip,
}

impl StripTags {
	/// Create a new [`StripTags`] node
	pub fn new(
		_ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.is_empty() {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			strip: FlacMetaStrip::new(),
			data: ConnectedInput::NotConnected,
		})
	}
}

// Input: "data" - Blob
// Output: "out" - Blob
impl Node<PipeData, CopperContext> for StripTags {
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

			OpenBytesSourceReader::S3(r) => {
				let mut v = Vec::new();
				r.take(ctx.blob_fragment_size).read(&mut v).unwrap();
				self.strip
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				if r.is_done() {
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
					PortName::new("out"),
					PipeData::Blob {
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
