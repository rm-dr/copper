use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use async_trait::async_trait;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	helpers::BytesSourceReader,
	CopperContext, JobRunResult,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, trace};

/// Extract tags from audio metadata
pub struct ExtractTags {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl<Itemdb: ItemdbClient> Node<JobRunResult, PipeData, CopperContext<Itemdb>> for ExtractTags {
	async fn run(
		&self,
		ctx: &CopperContext<Itemdb>,
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
		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}

		trace!(
			message = "Inputs ready, preparing reader",
			node_id = ?this_node.id
		);

		let mut reader = match data.unwrap() {
			None => {
				return Err(RunNodeError::RequiredInputNull {
					port: PortName::new("data"),
				})
			}

			Some(PipeData::Blob { source, .. }) => BytesSourceReader::open(ctx, source).await?,

			_ => {
				return Err(RunNodeError::BadInputType {
					port: PortName::new("data"),
				})
			}
		};

		//
		// Setup is done, extract tags
		//
		debug!(
			message = "Extracting tags",
			node_id = ?this_node.id
		);

		let mut block_reader = FlacBlockReader::new(FlacBlockSelector {
			pick_vorbiscomment: true,
			..Default::default()
		});

		while let Some(data) = reader.next_fragment(ctx.blob_fragment_size).await? {
			block_reader
				.push_data(&data)
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
		}

		block_reader
			.finish()
			.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

		//
		// Return tags
		//
		while block_reader.has_block() {
			let b = block_reader.pop_block().unwrap();
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
			assert!(!block_reader.has_block());
		}

		return Ok(());
	}
}
