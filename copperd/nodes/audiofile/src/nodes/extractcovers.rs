use crate::flac::proc::pictures::FlacPictureReader;
use async_trait::async_trait;
use copper_piper::{
	base::{Node, NodeBuilder, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	helpers::{processor::BytesProcessorBuilder, rawbytes::RawBytesSource, NodeParameters},
	CopperContext,
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, trace};

pub struct ExtractCovers {}

impl NodeBuilder for ExtractCovers {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>> {
		Box::new(Self {})
	}
}

// Inputs: "data" - Bytes
#[async_trait]
impl<'ctx> Node<'ctx> for ExtractCovers {
	async fn run(
		&self,
		ctx: &CopperContext<'ctx>,
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
		// Setup is done, extract covers
		//
		debug!(
			message = "Extracting covers",
			node_id = ?this_node.id
		);
		let mut picreader = FlacPictureReader::new();

		while let Some(data) = reader.next_fragment().await? {
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

		let mut output = BTreeMap::new();

		if let Some(picture) = picreader.pop_picture() {
			debug!(
				message = "Found a cover, sending",
				node_id = ?this_node.id,
				picture = ?picture
			);

			output.insert(
				PortName::new("cover_data"),
				PipeData::Blob {
					source: BytesProcessorBuilder::new(RawBytesSource::Array {
						mime: picture.mime.clone(),
						data: Arc::new(picture.img_data),
					}),
				},
			);
		} else {
			debug!(
				message = "Did not find a cover, sending None",
				node_id = ?this_node.id
			);
		}

		return Ok(output);
	}
}
