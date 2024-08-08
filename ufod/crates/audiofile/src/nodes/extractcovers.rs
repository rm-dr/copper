use crate::flac::proc::pictures::FlacPictureReader;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};
use ufo_node_base::{
	data::{BytesSource, UFOData, UFODataStub},
	helpers::DataSource,
	UFOContext,
};
use ufo_pipeline::{
	api::{InitNodeError, Node, NodeInfo, NodeState, PipelineData, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use ufo_util::mime::MimeType;

/// Info for a [`ExtractCovers`] node
pub struct ExtractCoversInfo {
	inputs: BTreeMap<PipelinePortID, UFODataStub>,
	outputs: BTreeMap<PipelinePortID, UFODataStub>,
}

impl ExtractCoversInfo {
	/// Generate node info from parameters
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 0 {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			inputs: BTreeMap::from([(PipelinePortID::new("data"), UFODataStub::Bytes)]),
			outputs: BTreeMap::from([(PipelinePortID::new("cover_data"), UFODataStub::Bytes)]),
		})
	}
}

impl NodeInfo<UFOData> for ExtractCoversInfo {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <UFOData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <UFOData as PipelineData>::DataStubType> {
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
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, InitNodeError> {
		Ok(Self {
			info: ExtractCoversInfo::new(params)?,
			blob_fragment_size: ctx.blob_fragment_size,
			reader: FlacPictureReader::new(),
			data: DataSource::Uninitialized,
		})
	}
}

impl Node<UFOData> for ExtractCovers {
	fn get_info(&self) -> &dyn ufo_pipeline::api::NodeInfo<UFOData> {
		&self.info
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: UFOData,
	) -> Result<(), RunNodeError> {
		match target_port.id().as_str() {
			"data" => match input_data {
				UFOData::Bytes { source, mime } => {
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
		send_data: &dyn Fn(PipelinePortID, UFOData) -> Result<(), RunNodeError>,
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

			DataSource::File { file, .. } => {
				let mut v = Vec::new();
				let n = file
					.by_ref()
					.take(self.blob_fragment_size)
					.read_to_end(&mut v)?;
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
		// TODO: send an array of covers
		if let Some(picture) = self.reader.pop_picture() {
			send_data(
				PipelinePortID::new("cover_data"),
				UFOData::Bytes {
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
				UFOData::None {
					data_type: UFODataStub::Bytes,
				},
			)?;
			return Ok(NodeState::Done);
		}

		return Ok(NodeState::Pending("No pictures yet"));
	}
}