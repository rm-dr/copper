use crate::flac::proc::pictures::FlacPictureReader;
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::{BytesSource, PipeData},
	helpers::OpenBytesSourceReader,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, trace, warn};

pub struct ExtractCovers {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl Node<PipeData, CopperContext> for ExtractCovers {
	async fn run(
		&self,
		ctx: &CopperContext,
		this_node: ThisNodeInfo,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
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
				BytesSource::Array { data, .. } => OpenBytesSourceReader::Array(data),
				BytesSource::Stream { receiver, .. } => OpenBytesSourceReader::Stream(receiver),

				BytesSource::S3 { key } => OpenBytesSourceReader::S3(
					ctx.objectstore_client
						.create_reader(&key)
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?,
				),
			},

			_ => {
				return Err(RunNodeError::BadInputType {
					port: PortName::new("data"),
				})
			}
		};

		let mut reader = FlacPictureReader::new();

		debug!(
			message = "Setup done, extracting covers",
			node_id = ?this_node.id
		);

		//
		// Setup is done, extract covers
		//
		match data {
			OpenBytesSourceReader::Array(data) => {
				trace!(
					message = "Reading data from array",
					node_id = ?this_node.id
				);

				reader
					.push_data(&data)
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

				reader
					.finish()
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
			}

			OpenBytesSourceReader::Stream(mut receiver) => {
				trace!(
					message = "Reading data from stream",
					node_id = ?this_node.id
				);

				loop {
					let rec = receiver.recv().await;
					match rec {
						Ok(d) => {
							reader
								.push_data(&d.data)
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

							if d.is_last {
								break;
							}
						}

						Err(broadcast::error::RecvError::Lagged(_)) => {
							return Err(RunNodeError::StreamReceiverLagged)
						}

						Err(broadcast::error::RecvError::Closed) => {
							warn!(
								message = "Receiver was closed before receiving last packet",
								node_id = ?this_node.id,
								node_type = ?this_node.node_type
							);
							break;
						}
					}
				}

				reader
					.finish()
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
			}

			OpenBytesSourceReader::S3(mut r) => {
				trace!(
					message = "Reading data from S3",
					node_id = ?this_node.id
				);

				let mut read_buf = vec![0u8; ctx.blob_fragment_size];

				loop {
					let l = r
						.read(&mut read_buf)
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					trace!(
						message = "Got bytes from S3",
						n_bytes = l,
						node_id = ?this_node.id
					);

					if l == 0 {
						assert!(r.is_done());
						break;
					} else {
						reader
							.push_data(&read_buf[0..l])
							.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
					}
				}

				trace!(
					message = "Reader ran out of data, finishing",
					node_id = ?this_node.id
				);

				assert!(r.is_done());
				reader
					.finish()
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
			}
		}

		//
		// Send the first cover we find
		//
		if let Some(picture) = reader.pop_picture() {
			debug!(
				message = "Found a cover, sending",
				node_id = ?this_node.id,
				picture = ?picture
			);

			output
				.send(NodeOutput {
					node: this_node,
					port: PortName::new("cover_data"),
					data: Some(PipeData::Blob {
						source: BytesSource::Array {
							mime: picture.mime.clone(),
							data: Arc::new(picture.img_data),
						},
					}),
				})
				.await?;
		} else {
			debug!(
				message = "Did not find a cover, sending None",
				node_id = ?this_node.id
			);

			output
				.send(NodeOutput {
					node: this_node,
					port: PortName::new("cover_data"),
					data: None,
				})
				.await?;
		}

		return Ok(());
	}
}
