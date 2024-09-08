use crate::flac::proc::pictures::FlacPictureReader;
use copper_util::mime::MimeType;
use pipelined_node_base::{
	data::{BytesSource, CopperData, CopperDataStub},
	helpers::DataSource,
	CopperContext,
};
use pipelined_pipeline::{
	api::{InitNodeError, Node, NodeInfo, NodeState, PipelineData, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Info for a [`ExtractCovers`] node
pub struct ExtractCoversInfo {
	inputs: BTreeMap<PipelinePortID, CopperDataStub>,
	outputs: BTreeMap<PipelinePortID, CopperDataStub>,
}

impl ExtractCoversInfo {
	/// Generate node info from parameters
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.is_empty() {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			inputs: BTreeMap::from([(PipelinePortID::new("data"), CopperDataStub::Bytes)]),
			outputs: BTreeMap::from([(PipelinePortID::new("cover_data"), CopperDataStub::Bytes)]),
		})
	}
}

impl NodeInfo<CopperData> for ExtractCoversInfo {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.outputs
	}
}

/// Extract covers from an audio file
pub struct ExtractCovers {
	info: ExtractCoversInfo,
	blob_fragment_size: u64,
	data: DataSource,
	reader: FlacPictureReader,
}

impl ExtractCovers {
	/// Create a new [`ExtractCovers`] node
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		Ok(Self {
			info: ExtractCoversInfo::new(params)?,
			blob_fragment_size: ctx.blob_fragment_size,
			reader: FlacPictureReader::new(),
			data: DataSource::Uninitialized,
		})
	}
}

impl Node<CopperData> for ExtractCovers {
	fn get_info(&self) -> &dyn NodeInfo<CopperData> {
		&self.info
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: CopperData,
	) -> Result<(), RunNodeError> {
		match target_port.id().as_str() {
			"data" => match input_data {
				CopperData::Bytes { source, mime } => {
					if mime != MimeType::Flac {
						return Err(RunNodeError::UnsupportedFormat(format!(
							"cannot extract covers from `{}`",
							mime
						)));
					}

					self.data.consume(mime, source);
				}

				_ => unreachable!("Unexpected input type"),
			},

			_ => unreachable!("Received data at invalid port"),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		// Push latest data into cover reader
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(NodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
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

			DataSource::Url { data, .. } => {
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
				PipelinePortID::new("cover_data"),
				CopperData::Bytes {
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
				PipelinePortID::new("cover_data"),
				CopperData::None {
					data_type: CopperDataStub::Bytes,
				},
			)?;
			return Ok(NodeState::Done);
		}

		return Ok(NodeState::Pending("No pictures yet"));
	}
}
