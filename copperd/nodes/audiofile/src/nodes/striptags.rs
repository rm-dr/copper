//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::{BytesSource, PipeData},
	helpers::OpenBytesSourceReader,
	CopperContext,
};
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, trace};

/// Strip all metadata from an audio file
pub struct StripTags {}

// Input: "data" - Blob
// Output: "out" - Blob
#[async_trait]
impl Node<PipeData, CopperContext> for StripTags {
	async fn run(
		&self,
		ctx: &CopperContext,
		this_node: ThisNodeInfo,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, Option<PipeData>>,
		output: mpsc::Sender<NodeOutput<PipeData>>,
	) -> Result<(), RunNodeError<PipeData>> {
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
		let data = match data.unwrap() {
			None => {
				return Err(RunNodeError::RequiredInputNull {
					port: PortName::new("data"),
				})
			}

			Some(PipeData::Blob { source, .. }) => match source {
				BytesSource::Stream { receiver, .. } => OpenBytesSourceReader::Array(receiver),
				BytesSource::S3 { key } => {
					OpenBytesSourceReader::S3(ctx.objectstore_client.create_reader(&key).await)
				}
			},
			_ => {
				return Err(RunNodeError::BadInputType {
					port: PortName::new("data"),
				})
			}
		};

		//
		// Send output handle
		//
		let (tx, rx) = broadcast::channel(ctx.stream_channel_capacity);
		output
			.send(NodeOutput {
				node: this_node.clone(),
				port: PortName::new("out"),
				data: Some(PipeData::Blob {
					mime: MimeType::Flac,
					source: BytesSource::Stream {
						sender: tx.clone(),
						receiver: rx,
					},
				}),
			})
			.await?;

		debug!(
			message = "Setup done, stripping tags",
			node_id = ?this_node.id
		);

		//
		// Strip tags
		//
		let mut strip = FlacMetaStrip::new();

		match data {
			OpenBytesSourceReader::Array(mut receiver) => {
				trace!(
					message = "Reading data from array",
					node_id = ?this_node.id
				);

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
						strip
							.read_data(&mut out_bytes)
							.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
					}

					if out_bytes.len() >= ctx.blob_fragment_size {
						let x = std::mem::replace(&mut out_bytes, Vec::new());
						tx.send(Arc::new(x))?;
					}
				}

				strip
					.finish()
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

				while strip.has_data() {
					strip
						.read_data(&mut out_bytes)
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
				}

				tx.send(Arc::new(out_bytes))?;
			}

			OpenBytesSourceReader::S3(mut r) => {
				trace!(
					message = "Reading data from S3",
					node_id = ?this_node.id
				);

				let mut out_bytes = Vec::new();
				let mut read_buf = vec![0u8; ctx.blob_fragment_size];

				loop {
					let l = r.read(&mut read_buf).await?;
					trace!(
						message = "Got bytes from S3",
						n_bytes = l,
						node_id = ?this_node.id
					);

					if l == 0 {
						assert!(r.is_done());
						break;
					} else {
						strip
							.push_data(&read_buf[0..l])
							.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
					}

					if strip.has_data() {
						strip
							.read_data(&mut out_bytes)
							.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
					}

					if out_bytes.len() >= ctx.blob_fragment_size {
						let x = std::mem::replace(&mut out_bytes, Vec::new());
						debug!(
							message = "Sending bytes",
							n_bytes = x.len(),
							node_id = ?this_node.id
						);
						tx.send(Arc::new(x))?;
					}
				}

				trace!(
					message = "Reader ran out of data, finishing",
					node_id = ?this_node.id
				);

				strip
					.finish()
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
				while strip.has_data() {
					strip
						.read_data(&mut out_bytes)
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
				}

				debug!(
					message = "Sending final bytes",
					n_bytes = out_bytes.len(),
					node_id = ?this_node.id
				);
				tx.send(Arc::new(out_bytes))?;
			}
		}

		return Ok(());
	}
}
