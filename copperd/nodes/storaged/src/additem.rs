use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	helpers::BytesSourceReader,
	CopperContext, JobRunResult,
};
use copper_storaged::{AttrData, AttributeInfo, ClassInfo, ResultOrDirect, TransactionAction};
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, trace};

pub struct AddItem {}

// Inputs: depends on class
// Outputs: None
#[async_trait]
impl Node<JobRunResult, PipeData, CopperContext> for AddItem {
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
		let class: ClassInfo = if let Some(value) = params.remove("class") {
			match value {
				NodeParameterValue::Integer(x) => ctx
					.storaged_client
					.get_class(x.into())
					.await
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?
					.ok_or(RunNodeError::BadParameterOther {
						parameter: "class".into(),
						message: "this class doesn't exist".into(),
					})?,

				_ => {
					return Err(RunNodeError::BadParameterType {
						parameter: "class".into(),
					})
				}
			}
		} else {
			return Err(RunNodeError::MissingParameter {
				parameter: "class".into(),
			});
		};

		// This is only used by UI, but make sure it's sane.

		if let Some(value) = params.remove("dataset") {
			match value {
				NodeParameterValue::Integer(x) => {
					let dataset = ctx
						.storaged_client
						.get_dataset(x.into())
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?
						.ok_or(RunNodeError::BadParameterOther {
							parameter: "dataset".into(),
							message: "this dataset doesn't exist".into(),
						})?;

					if class.dataset != dataset.id {
						return Err(RunNodeError::BadParameterOther {
							parameter: "dataset".into(),
							message: "this class doesn't belong to this dataset".into(),
						});
					}

					if ctx.run_by_user != dataset.owner {
						return Err(RunNodeError::NotAuthorized {
							message: "you do not have permission to modify this dataset".into(),
						});
					}
				}

				_ => {
					return Err(RunNodeError::BadParameterType {
						parameter: "dataset".into(),
					})
				}
			}
		} else {
			return Err(RunNodeError::MissingParameter {
				parameter: "dataset".into(),
			});
		};
		if let Some((param, _)) = params.first_key_value() {
			return Err(RunNodeError::UnexpectedParameter {
				parameter: param.clone(),
			});
		}

		//
		// Set up attribute table
		// (extract inputs)
		//

		// TODO: map 404s to "no class" errors
		debug!(message = "Getting attributes");
		let mut attributes: BTreeMap<PortName, (AttributeInfo, Option<ResultOrDirect<AttrData>>)> =
			class
				.attributes
				.into_iter()
				.map(|x| (PortName::new(x.name.as_str()), (x, None)))
				.collect();

		// Fill attribute table
		while let Some((port, data)) = input.pop_first() {
			trace!(message = "Receiving data from port", ?port);

			if !attributes.contains_key(&port) {
				return Err(RunNodeError::UnrecognizedInput { port });
			}

			match data {
				Some(PipeData::Blob { source }) => {
					// TODO: recompute if exists
					let new_obj_key: SmartString<LazyCompact> = rand::thread_rng()
						.sample_iter(&Alphanumeric)
						.take(32)
						.map(char::from)
						.collect();

					let mut part_counter = 1;

					let mut reader = BytesSourceReader::open(ctx, source).await?;

					let mut upload = ctx
						.objectstore_client
						.create_multipart_upload(
							&ctx.objectstore_blob_bucket,
							&new_obj_key,
							reader.mime().clone(),
						)
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					while let Some(data) = reader.next_fragment(ctx.blob_fragment_size).await? {
						upload
							.upload_part(&data, part_counter)
							.await
							.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
						part_counter += 1;
					}

					upload
						.finish()
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					let attr = attributes.get_mut(&port).unwrap();

					attr.1 = Some(
						AttrData::Blob {
							bucket: ctx.objectstore_blob_bucket.clone(),
							key: new_obj_key,
						}
						.into(),
					)
				}

				Some(PipeData::TransactionActionResult {
					action_idx,
					result_type,
				}) => {
					let attr = attributes.get_mut(&port).unwrap();

					if result_type != attr.0.data_type {
						return Err(RunNodeError::BadInputType { port });
					}

					attr.1 = Some(ResultOrDirect::Result {
						action_idx,
						expected_type: result_type,
					});
				}

				Some(x) => {
					let attr = attributes.get_mut(&port).unwrap();
					let as_attr: AttrData = match x.try_into() {
						Ok(x) => x,
						Err(_) => return Err(RunNodeError::BadInputType { port }),
					};

					if as_attr.as_stub() != attr.0.data_type {
						return Err(RunNodeError::BadInputType { port });
					}

					attr.1 = Some(as_attr.into());
				}

				None => {}
			};
		}

		//
		// Set up and send transaction
		//

		let action = TransactionAction::AddItem {
			to_class: class.id,
			attributes: attributes
				.into_iter()
				.map(|(_, (k, d))| {
					(
						k.id,
						d.unwrap_or(
							AttrData::None {
								data_type: k.data_type,
							}
							.into(),
						),
					)
				})
				.collect(),
		};

		let mut trans = ctx.transaction.lock().await;
		let result_type = action.result_type().unwrap();
		debug!(
			message = "Registering action",
			?action,
			action_idx = trans.len()
		);
		let action_idx = trans.add_action(action);

		output
			.send(NodeOutput {
				node: this_node,
				port: PortName::new("new_item"),
				data: Some(PipeData::TransactionActionResult {
					action_idx,
					result_type,
				}),
			})
			.await?;

		return Ok(());
	}
}
