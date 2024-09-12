//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeParameterValue, PortName, RunNodeError},
	data::{BytesSource, PipeData},
	helpers::{BytesSourceArrayReader, OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

/// Strip all metadata from an audio file
pub struct StripTags {}

// Input: "data" - Blob
// Output: "out" - Blob
#[async_trait]
impl Node<PipeData, CopperContext> for StripTags {
	async fn run(
		&self,
		ctx: &CopperContext,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, PipeData>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
		//
		// Extract parameters
		//
		if let Some((param, _)) = params.first_key_value() {
			return Err(RunNodeError::UnexpectedParameter {
				parameter: param.clone(),
			});
		}

		//
		// Extract arguments
		//
		let data = input.remove(&PortName::new("data"));
		if data.is_none() {
			return Err(RunNodeError::MissingInput {
				port: PortName::new("data"),
			});
		}
		let mut data = match data {
			None => unreachable!(),
			Some(PipeData::Blob { mime, source }) => match source {
				BytesSource::Array { .. } => OpenBytesSourceReader::Array(
					BytesSourceArrayReader::new(Some(mime), source).unwrap(),
				),

				BytesSource::S3 { key } => OpenBytesSourceReader::S3(
					S3Reader::new(ctx.objectstore_client.clone(), &ctx.objectstore_bucket, key)
						.await,
				),
			},
			_ => {
				return Err(RunNodeError::BadInputType {
					port: PortName::new("data"),
				})
			}
		};

		let mut strip = FlacMetaStrip::new();

		//
		// Setup is done, strip tags
		//
		match &mut data {
			OpenBytesSourceReader::Array(BytesSourceArrayReader { data, is_done, .. }) => {
				while let Some(d) = data.pop_front() {
					strip
						.push_data(&d)
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
				if *is_done {
					strip
						.finish()
						.map_err(|e| RunNodeError::Other(Box::new(e)))?;
				}
			}

			OpenBytesSourceReader::S3(r) => {
				let mut v = Vec::new();
				r.read_to_end(&mut v).unwrap();
				strip
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;

				assert!(r.is_done());

				strip
					.finish()
					.map_err(|e| RunNodeError::Other(Box::new(e)))?;
			}
		}

		//
		// Read and send stripped data
		//
		let mut out = BTreeMap::new();
		let mut bytes = Vec::new();

		while strip.has_data() {
			strip.read_data(&mut bytes).unwrap();
		}

		// TODO: do not load into memory
		out.insert(
			PortName::new("out"),
			PipeData::Blob {
				mime: MimeType::Flac,
				source: BytesSource::Array {
					fragment: Arc::new(bytes),
					is_last: true,
				},
			},
		);

		return Ok(out);
	}
}
