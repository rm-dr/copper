use crate::flac::proc::pictures::FlacPictureReader;
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError},
	data::{BytesSource, PipeData},
	helpers::{BytesSourceArrayReader, OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, io::Read, sync::Arc};

pub struct ExtractCovers {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl Node<PipeData, CopperContext> for ExtractCovers {
	async fn run(
		&self,
		ctx: &CopperContext,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, NodeOutput<PipeData>>,
	) -> Result<BTreeMap<PortName, NodeOutput<PipeData>>, RunNodeError> {
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
		let mut data = match data.unwrap().get_value().await? {
			None => {
				return Err(RunNodeError::RequiredInputNull {
					port: PortName::new("data"),
				})
			}

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

		let mut reader = FlacPictureReader::new();

		//
		// Setup is done, extract covers
		//
		match &mut data {
			OpenBytesSourceReader::Array(BytesSourceArrayReader { data, is_done, .. }) => {
				while let Some(d) = data.pop_front() {
					reader
						.push_data(&d)
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
				}
				if *is_done {
					reader
						.finish()
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
				}
			}

			OpenBytesSourceReader::S3(r) => {
				let mut v = Vec::new();
				r.take(ctx.blob_fragment_size).read_to_end(&mut v).unwrap();
				reader
					.push_data(&v)
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

				if r.is_done() {
					reader
						.finish()
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
				}
			}
		}

		//
		// Send the first cover we find
		//
		let mut out = BTreeMap::new();
		if let Some(picture) = reader.pop_picture() {
			out.insert(
				PortName::new("cover_data"),
				NodeOutput::Plain(Some(PipeData::Blob {
					mime: picture.mime.clone(),
					source: BytesSource::Array {
						fragment: Arc::new(picture.img_data),
						is_last: true,
					},
				})),
			);
		}

		return Ok(out);
	}
}
