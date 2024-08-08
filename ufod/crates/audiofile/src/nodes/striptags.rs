//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};
use ufo_node_base::{
	data::{BytesSource, UFOData, UFODataStub},
	helpers::DataSource,
	UFOContext,
};
use ufo_pipeline::{
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineNode, PipelineNodeError,
		PipelineNodeState,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use ufo_util::mime::MimeType;

/// Strip all metadata from an audio file
pub struct StripTags {
	inputs: Vec<NodeInputInfo<<UFOData as PipelineData>::DataStubType>>,
	outputs: Vec<NodeOutputInfo<<UFOData as PipelineData>::DataStubType>>,

	blob_fragment_size: u64,
	data: DataSource,
	strip: FlacMetaStrip,
}

impl StripTags {
	/// Create a new [`StripTags`] node
	pub fn new(
		ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, PipelineNodeError> {
		if params.len() != 0 {
			return Err(PipelineNodeError::BadParameterCount { expected: 0 });
		}

		Ok(Self {
			inputs: vec![NodeInputInfo {
				name: PipelinePortID::new("data"),
				accepts_type: UFODataStub::Bytes,
			}],

			outputs: vec![NodeOutputInfo {
				name: PipelinePortID::new("out"),
				produces_type: UFODataStub::Bytes,
			}],

			blob_fragment_size: ctx.blob_fragment_size,

			strip: FlacMetaStrip::new(),
			data: DataSource::Uninitialized,
		})
	}
}

impl PipelineNode<UFOData> for StripTags {
	fn inputs(&self) -> &[NodeInputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.inputs
	}

	fn outputs(&self) -> &[NodeOutputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.outputs
	}

	fn take_input(
		&mut self,
		target_port: usize,
		input_data: UFOData,
	) -> Result<(), ufo_pipeline::api::PipelineNodeError> {
		match target_port {
			0 => match input_data {
				UFOData::Bytes { source, mime } => {
					if mime != MimeType::Flac {
						return Err(PipelineNodeError::UnsupportedFormat(format!(
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
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<ufo_pipeline::api::PipelineNodeState, PipelineNodeError> {
		// Push latest data into metadata stripper
		match &mut self.data {
			DataSource::Uninitialized => {
				return Ok(PipelineNodeState::Pending("No data received"));
			}

			DataSource::Binary { data, is_done, .. } => {
				while let Some(d) = data.pop_front() {
					self.strip
						.push_data(&d)
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					self.strip
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
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
					.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;

				if n == 0 {
					self.strip
						.finish()
						.map_err(|e| PipelineNodeError::Other(Box::new(e)))?;
				}
			}
		}

		// Read and send stripped data
		if self.strip.has_data() {
			let mut out = Vec::new();
			self.strip.read_data(&mut out).unwrap();

			if !out.is_empty() {
				send_data(
					0,
					UFOData::Bytes {
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
			return Ok(PipelineNodeState::Done);
		} else {
			return Ok(PipelineNodeState::Pending("Waiting for more data"));
		}
	}
}
