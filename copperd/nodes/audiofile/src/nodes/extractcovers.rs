use crate::flac::proc::pictures::FlacPictureReader;
use async_trait::async_trait;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_piper::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::{BytesSource, PipeData},
	helpers::BytesSourceReader,
	CopperContext, JobRunResult,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, trace};

pub struct ExtractCovers {}

// Inputs: "data" - Bytes
// Outputs: variable, depends on tags
#[async_trait]
impl<Itemdb: ItemdbClient> Node<JobRunResult, PipeData, CopperContext<Itemdb>> for ExtractCovers {
	async fn run(
		&self,
		ctx: &CopperContext<Itemdb>,
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
		// Setup is done, extract covers
		//
		debug!(
			message = "Extracting covers",
			node_id = ?this_node.id
		);
		let mut picreader = FlacPictureReader::new();

		while let Some(data) = reader.next_fragment(ctx.blob_fragment_size).await? {
			picreader
				.push_data(&data)
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
		}

		picreader
			.finish()
			.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

		//
		// Send the first cover we find
		//
		if let Some(picture) = picreader.pop_picture() {
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
