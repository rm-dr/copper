use crate::{
	common::tagtype::TagType,
	flac::blockread::{FlacBlock, FlacBlockReader, FlacBlockSelector},
};
use async_trait::async_trait;
use copper_piper::{
	base::{Node, NodeBuilder, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	helpers::NodeParameters,
	CopperContext,
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, trace};

/// Extract tags from audio metadata
pub struct ExtractTags {}

impl NodeBuilder for ExtractTags {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>> {
		Box::new(Self {})
	}
}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl<'ctx> Node<'ctx> for ExtractTags {
	async fn run(
		&self,
		ctx: &CopperContext<'ctx>,
		this_node: ThisNodeInfo,
		mut params: NodeParameters,
		mut input: BTreeMap<PortName, Option<PipeData>>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
		//
		// Extract parameters
		//

		let tags = {
			let mut tags: BTreeMap<PortName, TagType> = BTreeMap::new();
			let val = params.pop_val("tags")?;

			match val {
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
			};

			tags
		};

		params.err_if_not_empty()?;

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

			Some(PipeData::Blob { source, .. }) => source.build(ctx).await?,

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

		while let Some(data) = reader.next_fragment().await? {
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

		let mut output = BTreeMap::new();

		while block_reader.has_block() {
			let b = block_reader.pop_block().unwrap();
			match b {
				FlacBlock::VorbisComment(comment) => {
					for (port, tag_type) in tags.iter() {
						if let Some((_, tag_value)) =
							comment.comment.comments.iter().find(|(t, _)| t == tag_type)
						{
							let x = output.insert(
								port.clone(),
								PipeData::Text {
									value: tag_value.clone(),
								},
							);

							// Each insertion should be new
							assert!(x.is_none());
						}
					}
				}

				// `reader` filters blocks for us
				_ => unreachable!(),
			}

			// We should only have one comment block
			assert!(!block_reader.has_block());
		}

		return Ok(output);
	}
}
