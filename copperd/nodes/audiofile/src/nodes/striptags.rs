//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError},
	data::{BytesSource, PipeData},
	helpers::{OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::broadcast;

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
		let data = match data.unwrap().get_value().await? {
			None => {
				return Err(RunNodeError::RequiredInputNull {
					port: PortName::new("data"),
				})
			}

			Some(PipeData::Blob { source, .. }) => match source {
				BytesSource::Stream { receiver, .. } => OpenBytesSourceReader::Array(receiver),
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

		//
		// Prepare stream output
		//
		let mut out = BTreeMap::new();
		let (tx, rx) = broadcast::channel(10);
		out.insert(
			PortName::new("out"),
			NodeOutput::Plain(Some(PipeData::Blob {
				mime: MimeType::Flac,
				source: BytesSource::Stream {
					sender: tx.clone(),
					receiver: rx,
				},
			})),
		);

		//
		// Start another task to strip tags
		//
		let h = tokio::spawn(async move {
			let mut strip = FlacMetaStrip::new();

			match data {
				OpenBytesSourceReader::Array(mut receiver) => {
					let mut out_bytes = Vec::new();

					loop {
						let rec = receiver.recv().await;
						match rec {
							Ok(d) => {
								strip
									.push_data(&d)
									.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
							}

							Err(broadcast::error::RecvError::Lagged(_)) => {
								return Err(RunNodeError::StreamReceiverLagged)
							}

							Err(broadcast::error::RecvError::Closed) => {
								break;
							}
						}

						while strip.has_data() {
							strip.read_data(&mut out_bytes).unwrap();
						}

						if out_bytes.len() >= 1000 {
							// TODO: config sizes
							// TODO: no unwrap
							// TODO: return tasks
							let x = std::mem::replace(&mut out_bytes, Vec::new());
							tx.send(Arc::new(x)).unwrap();
						}
					}

					strip
						.finish()
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					while strip.has_data() {
						strip.read_data(&mut out_bytes).unwrap();
					}

					tx.send(Arc::new(out_bytes)).unwrap();
				}

				OpenBytesSourceReader::S3(mut r) => {
					let mut out_bytes = Vec::new();
					let mut buf = [0u8; 1_000_000];

					loop {
						let l = r.read(&mut buf).await?;

						if l == 0 {
							assert!(r.is_done());
							break;
						} else {
							strip
								.push_data(&buf[0..l])
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
						}

						while strip.has_data() {
							strip.read_data(&mut out_bytes).unwrap();
						}

						if out_bytes.len() >= 1000 {
							let x = std::mem::replace(&mut out_bytes, Vec::new());
							tx.send(Arc::new(x)).unwrap();
						}
					}

					strip
						.finish()
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					while strip.has_data() {
						strip.read_data(&mut out_bytes).unwrap();
					}

					tx.send(Arc::new(out_bytes)).unwrap();
				}
			}

			Ok(())
		});

		return Ok(out);
	}
}
