//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use copper_node_base::{
	data::{BytesSource, CopperData, CopperDataStub},
	helpers::DataSource,
	CopperContext,
};
use copper_pipeline::{
	api::{InitNodeError, Node, NodeInfo, NodeState, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use copper_util::mime::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Info for a [`StripTags`] node
pub struct StripTagsInfo {
	inputs: BTreeMap<PipelinePortID, CopperDataStub>,
	outputs: BTreeMap<PipelinePortID, CopperDataStub>,
}

impl StripTagsInfo {
	/// Generate node info from parameters
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 0 {
			return Err(InitNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			inputs: BTreeMap::from([(PipelinePortID::new("data"), CopperDataStub::Bytes)]),
			outputs: BTreeMap::from([(PipelinePortID::new("out"), CopperDataStub::Bytes)]),
		})
	}
}

impl NodeInfo<CopperData> for StripTagsInfo {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, CopperDataStub> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, CopperDataStub> {
		&self.outputs
	}
}

/// Strip all metadata from an audio file
pub struct StripTags {
	info: StripTagsInfo,
	blob_fragment_size: u64,
	data: DataSource,
	strip: FlacMetaStrip,
}

impl StripTags {
	/// Create a new [`StripTags`] node
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
	) -> Result<Self, InitNodeError> {
		Ok(Self {
			info: StripTagsInfo::new(params)?,
			blob_fragment_size: ctx.blob_fragment_size,
			strip: FlacMetaStrip::new(),
			data: DataSource::Uninitialized,
		})
	}
}

impl Node<CopperData> for StripTags {
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
							"cannot strip tags from `{}`",
							mime
						)));
					}

					self.data.consume(mime, source);
				}

				_ => unreachable!("Received data with an unexpected type"),
			},

			_ => unreachable!("Received data at invalid port"),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		// Push latest data into metadata stripper
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(NodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
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

			DataSource::File { file, .. } => {
				let mut v = Vec::new();

				// Read in parts so we don't never have to load the whole
				// file into memory
				let n = file
					.by_ref()
					.take(self.blob_fragment_size)
					.read_to_end(&mut v)?;

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
