//! Strip all tags from an audio file

use crate::flac::proc::metastrip::FlacMetaStrip;
use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::{BytesSource, PipeData},
	helpers::BytesSourceReader,
	CopperContext, JobRunResult,
};
use copper_storage::database::base::client::StorageDatabaseClient;
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, trace};

/// Strip all metadata from an audio file
pub struct StripTags {}

// Input: "data" - Blob
// Output: "out" - Blob
#[async_trait]
impl<StorageClient: StorageDatabaseClient>
	Node<JobRunResult, PipeData, CopperContext<StorageClient>> for StripTags
{
	async fn run(
		&self,
		ctx: &CopperContext<StorageClient>,
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
		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}

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
		// Send output handle
		//
		let (tx, rx) = async_broadcast::broadcast(ctx.stream_channel_capacity);
		output
			.send(NodeOutput {
				node: this_node.clone(),
				port: PortName::new("out"),
				data: Some(PipeData::Blob {
					source: BytesSource::Stream {
						mime: MimeType::Flac,
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
		debug!(
			message = "Stripping tags",
			node_id = ?this_node.id
		);
		let mut strip = FlacMetaStrip::new();
		let mut out_bytes = Vec::new();

		while let Some(data) = reader.next_fragment(ctx.blob_fragment_size).await? {
			strip
				.push_data(&data)
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

			strip
				.read_data(&mut out_bytes)
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

			if out_bytes.len() >= ctx.blob_fragment_size {
				let x = std::mem::take(&mut out_bytes);
				trace!(
					message = "Sending bytes",
					n_bytes = x.len(),
					node_id = ?this_node.id
				);

				match tx.broadcast(Arc::new(x)).await {
					Ok(_) => {}

					// Exit early if no receivers exist
					Err(async_broadcast::SendError(_)) => {
						debug!(
							message = "Byte sender is closed, exiting early",
							node_id = ?this_node.id
						);
						return Ok(());
					}
				}
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

		trace!(
			message = "Sending final bytes",
			n_bytes = out_bytes.len(),
			node_id = ?this_node.id
		);

		match tx.broadcast(Arc::new(out_bytes)).await {
			Ok(_) => {}

			// Exit early if no receivers exist
			Err(async_broadcast::SendError(_)) => {
				debug!(
					message = "Byte sender is closed, exiting early",
					node_id = ?this_node.id
				);
				return Ok(());
			}
		}

		return Ok(());
	}
}
