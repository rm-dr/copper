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

pub struct ExtractCovers {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl Node<PipeData, CopperContext> for ExtractCovers {
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

		let mut reader = FlacPictureReader::new();

		//
		// Setup is done, extract covers
		//
		match data {
			OpenBytesSourceReader::Array(mut receiver) => {
				loop {
					let rec = receiver.recv().await;
					match rec {
						Ok(d) => {
							reader
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
				}

				reader
					.finish()
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
			}

			OpenBytesSourceReader::S3(mut r) => {
				let mut read_buf = vec![0u8; ctx.blob_fragment_size];

				loop {
					let l = r.read(&mut read_buf).await?;

					if l == 0 {
						assert!(r.is_done());
						break;
					} else {
						reader
							.push_data(&read_buf[0..l])
							.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
					}
				}

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
			// Pictures are loaded into memory anyway, so we don't need to stream them.
			let (tx, rx) = broadcast::channel(1);
			tx.send(Arc::new(picture.img_data))?;

			output
				.send(NodeOutput {
					node: this_node,
					port: PortName::new("cover_data"),
					data: Some(PipeData::Blob {
						mime: picture.mime.clone(),
						source: BytesSource::Stream {
							sender: tx.clone(),
							receiver: rx,
						},
					}),
				})
				.await?;
		} else {
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
