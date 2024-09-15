use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::{BytesSource, PipeData},
	CopperContext,
};
use copper_storaged::{AttrData, AttributeInfo, ClassId, ResultOrDirect, TransactionAction};
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, trace, warn};

pub struct AddItem {}

// Inputs: depends on class
// Outputs: None
#[async_trait]
impl Node<PipeData, CopperContext> for AddItem {
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
		let class: ClassId = if let Some(value) = params.remove("class") {
			match value {
				NodeParameterValue::Integer(x) => match u32::try_from(x) {
					Ok(x) => x.into(),
					Err(_) => {
						return Err(RunNodeError::BadParameterType {
							parameter: "class".into(),
						})
					}
				},
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
		if let Some((param, _)) = params.first_key_value() {
			return Err(RunNodeError::UnexpectedParameter {
				parameter: param.clone(),
			});
		}

		//
		// Set up attribute table
		// (extract inputs)
		//
		debug!(message = "Getting attributes");
		let mut attributes: BTreeMap<PortName, (AttributeInfo, Option<ResultOrDirect<AttrData>>)> =
			ctx.storaged_client
				.get_class(class)
				.await
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?
				.ok_or(RunNodeError::BadParameterOther {
					parameter: "class".into(),
					message: "this class doesn't exist".into(),
				})?
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
					let new_obj_key: String = rand::thread_rng()
						.sample_iter(&Alphanumeric)
						.take(32)
						.map(char::from)
						.collect();

					let mut part_counter = 1;

					let upload = match source {
						BytesSource::Array { mime, data } => {
							let mut upload = ctx
								.objectstore_client
								.create_multipart_upload(&new_obj_key, mime)
								.await
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

							upload
								.upload_part(&data, 1)
								.await
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

							upload
						}

						BytesSource::Stream {
							mut receiver, mime, ..
						} => {
							let mut upload = ctx
								.objectstore_client
								.create_multipart_upload(&new_obj_key, mime)
								.await
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

							loop {
								let rec = receiver.recv().await;

								match rec {
									Ok(d) => {
										upload
											.upload_part(&d.data, part_counter)
											.await
											.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
										part_counter += 1;
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

							upload
						}

						BytesSource::S3 { key } => {
							let mut reader = ctx
								.objectstore_client
								.create_reader(&key)
								.await
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

							let mut upload = ctx
								.objectstore_client
								.create_multipart_upload(&new_obj_key, reader.mime().clone())
								.await
								.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

							let mut read_buf = vec![0u8; ctx.blob_fragment_size];

							loop {
								let l = reader
									.read(&mut read_buf)
									.await
									.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

								if l != 0 {
									break;
								} else {
									upload
										.upload_part(&read_buf, part_counter)
										.await
										.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
									part_counter += 1;
								}
							}

							upload
						}
					};

					upload
						.finish()
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					let attr = attributes.get_mut(&port).unwrap();

					attr.1 = Some(
						AttrData::Blob {
							object_key: new_obj_key,
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
			to_class: class,
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
