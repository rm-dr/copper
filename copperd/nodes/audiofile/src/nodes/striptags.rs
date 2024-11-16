//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use async_trait::async_trait;
use copper_piper::{
	base::{Node, NodeBuilder, NodeId, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	helpers::{
		processor::{StreamProcessor, StreamProcessorBuilder},
		NodeParameters,
	},
	CopperContext,
};
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::debug;

/// Strip all metadata from an audio file
pub struct StripTags {}

impl NodeBuilder for StripTags {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>> {
		Box::new(Self {})
	}
}

// Input: "data" - Blob
// Output: "out" - Blob
#[async_trait]
impl<'ctx> Node<'ctx> for StripTags {
	async fn run(
		&self,
		_ctx: &CopperContext<'ctx>,
		this_node: ThisNodeInfo,
		params: NodeParameters,
		mut input: BTreeMap<PortName, Option<PipeData>>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
		//
		// Extract parameters
		//
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

		let source = match data.unwrap() {
			None => {
				return Err(RunNodeError::RequiredInputNull {
					port: PortName::new("data"),
				})
			}

			Some(PipeData::Blob { source, .. }) => source,

			_ => {
				return Err(RunNodeError::BadInputType {
					port: PortName::new("data"),
				})
			}
		};

		debug!(
			message = "Setup done, stripping tags",
			node_id = ?this_node.id
		);

		let mut output = BTreeMap::new();

		output.insert(
			PortName::new("out"),
			PipeData::Blob {
				source: source.add_processor(Arc::new(TagStripProcessor {
					node_id: this_node.id.clone(),
					node_type: this_node.node_type.clone(),
				})),
			},
		);

		return Ok(output);
	}
}

#[derive(Debug, Clone)]
struct TagStripProcessor {
	node_id: NodeId,
	node_type: SmartString<LazyCompact>,
}

impl StreamProcessorBuilder for TagStripProcessor {
	fn build(&self) -> Box<dyn StreamProcessor> {
		Box::new(self.clone())
	}
}

#[async_trait]
impl StreamProcessor for TagStripProcessor {
	fn mime(&self) -> &MimeType {
		return &MimeType::Flac;
	}

	fn name(&self) -> &'static str {
		"TagStripProcessor"
	}

	fn source_node_id(&self) -> &NodeId {
		&self.node_id
	}

	/// Return the type of the node that created this processor
	fn source_node_type(&self) -> &str {
		&self.node_type
	}

	async fn run(
		&self,
		mut source: Receiver<Arc<Vec<u8>>>,
		sink: Sender<Arc<Vec<u8>>>,
		max_buffer_size: usize,
	) -> Result<(), RunNodeError> {
		//
		// Strip tags
		//

		let mut strip = FlacMetaStrip::new();
		let mut out_bytes = Vec::new();

		while let Some(data) = source.recv().await {
			strip
				.push_data(&data)
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

			strip
				.read_data(&mut out_bytes)
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

			if out_bytes.len() >= max_buffer_size {
				let x = std::mem::take(&mut out_bytes);

				match sink.send(Arc::new(x)).await {
					Ok(()) => {}

					// Not an error, our receiver was dropped.
					// Exit early if that happens!
					Err(_) => return Ok(()),
				};
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

		match sink.send(Arc::new(out_bytes)).await {
			Ok(()) => {}

			// Not an error, our receiver was dropped.
			// Exit early if that happens!
			Err(_) => return Ok(()),
		};

		return Ok(());
	}
}
