use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError},
	data::{BytesSource, PipeData},
	helpers::{OpenBytesSourceReader, S3Reader},
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::broadcast;

/// Extract tags from audio metadata
pub struct ExtractTags {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl Node<PipeData, CopperContext> for ExtractTags {
	async fn run(
		&self,
		ctx: &CopperContext,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, NodeOutput<PipeData>>,
	) -> Result<BTreeMap<PortName, NodeOutput<PipeData>>, RunNodeError> {
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
				let mut buf = [0u8; 1_000_000];

				loop {
					let l = r.read(&mut buf).await?;

					if l == 0 {
						assert!(r.is_done());
						break;
					} else {
						reader
							.push_data(&buf[0..l])
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
		let mut out = BTreeMap::new();
		while reader.has_block() {
			let b = reader.pop_block().unwrap();
			match b {
				FlacBlock::VorbisComment(comment) => {
					for (port, tag_type) in tags.iter() {
						if let Some((_, tag_value)) =
							comment.comment.comments.iter().find(|(t, _)| t == tag_type)
						{
							out.insert(
								port.clone(),
								NodeOutput::Plain(Some(PipeData::Text {
									value: tag_value.clone(),
								})),
							);
						} else {
							out.insert(port.clone(), NodeOutput::Plain(None));
						}
					}
				}

				// `reader` filters blocks for us
				_ => unreachable!(),
			}

			// We should only have one comment block
			assert!(!reader.has_block());
		}

		return Ok(out);
	}
}
