use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
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
use tracing::warn;

/// Extract tags from audio metadata
pub struct ExtractTags {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl Node<PipeData, CopperContext> for ExtractTags {
	async fn run(
		&self,
		ctx: &CopperContext,
		this_node: ThisNodeInfo,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
		mut input: BTreeMap<PortName, Option<PipeData>>,
		output: mpsc::Sender<NodeOutput<PipeData>>,
	) -> Result<(), RunNodeError<PipeData>> {
		//
		// Extract parameters
		//
		let mut tags: BTreeMap<PortName, TagType> = BTreeMap::new();
		if let Some(taglist) = params.remove("tags") {
			match taglist {
				NodeParameterValue::List(list) => {
					for t in list {
						match t {
							NodeParameterValue::String(s) => {
								tags.insert(PortName::new(s.as_str()), s.as_str().into());
							}
							_ => {
								return Err(RunNodeError::BadParameterType {
									parameter: "tags".into(),
								})
							}
						}
					}
				}
				_ => {
					return Err(RunNodeError::BadParameterType {
						parameter: "tags".into(),
					})
				}
			}
		} else {
			return Err(RunNodeError::MissingParameter {
				parameter: "tags".into(),
			});
		}

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

		let mut reader = FlacBlockReader::new(FlacBlockSelector {
			pick_vorbiscomment: true,
			..Default::default()
		});

		//
		// Setup is done, extract tags
		//
		match data {
			OpenBytesSourceReader::Array(mut receiver) => {
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
				let mut read_buf = vec![0u8; ctx.blob_fragment_size];

				loop {
					let l = r
						.read(&mut read_buf)
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

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
		// Return tags
		//
		while reader.has_block() {
			let b = reader.pop_block().unwrap();
			match b {
				FlacBlock::VorbisComment(comment) => {
					for (port, tag_type) in tags.iter() {
						if let Some((_, tag_value)) =
							comment.comment.comments.iter().find(|(t, _)| t == tag_type)
						{
							output
								.send(NodeOutput {
									node: this_node.clone(),
									port: port.clone(),
									data: Some(PipeData::Text {
										value: tag_value.clone(),
									}),
								})
								.await?;
						} else {
							output
								.send(NodeOutput {
									node: this_node.clone(),
									port: port.clone(),
									data: None,
								})
								.await?;
						}
					}
				}

				// `reader` filters blocks for us
				_ => unreachable!(),
			}

			// We should only have one comment block
			assert!(!reader.has_block());
		}

		return Ok(());
	}
}
