use crate::flac::proc::pictures::FlacPictureReader;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};
use ufo_node_base::{
	data::{BytesSource, UFOData, UFODataStub},
	helpers::DataSource,
	UFOContext,
};
use ufo_pipeline::{
	api::{InitNodeError, NodeInfo, PipelineData, PipelineNode, PipelineNodeState, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use ufo_util::mime::MimeType;

/// Info for a [`ExtractCovers`] node
pub struct ExtractCoversInfo {
	inputs: [(PipelinePortID, UFODataStub); 1],
	outputs: [(PipelinePortID, UFODataStub); 1],
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
			inputs: [(PipelinePortID::new("data"), UFODataStub::Bytes)],
			outputs: [(PipelinePortID::new("cover_data"), UFODataStub::Bytes)],
		})
	}
}

impl NodeInfo<UFOData> for ExtractCoversInfo {
	fn inputs(&self) -> &[(PipelinePortID, <UFOData as PipelineData>::DataStubType)] {
		&self.inputs
	}

	fn outputs(&self) -> &[(PipelinePortID, <UFOData as PipelineData>::DataStubType)] {
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

impl PipelineNode<UFOData> for ExtractCovers {
	fn get_info(&self) -> &dyn ufo_pipeline::api::NodeInfo<UFOData> {
		&self.info
	}

	fn take_input(&mut self, target_port: usize, input_data: UFOData) -> Result<(), RunNodeError> {
		match target_port {
			0 => match input_data {
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
		send_data: &dyn Fn(usize, UFOData) -> Result<(), RunNodeError>,
	) -> Result<PipelineNodeState, RunNodeError> {
		// Push latest data into cover reader
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(PipelineNodeState::Pending("No data received"));
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
				0,
				UFOData::Bytes {
					mime: picture.mime.clone(),
					source: BytesSource::Array {
						fragment: Arc::new(picture.img_data),
						is_last: true,
					},
				},
			)?;
			return Ok(PipelineNodeState::Done);
		} else if self.reader.is_done() {
			send_data(
				0,
				UFOData::None {
					data_type: UFODataStub::Bytes,
				},
			)?;
			return Ok(PipelineNodeState::Done);
		}

		return Ok(PipelineNodeState::Pending("No pictures yet"));
	}
}
