use async_trait::async_trait;
use copper_itemdb::{
	client::errors::{class::GetClassError, dataset::GetDatasetError},
	AttrData, AttributeInfo, ClassInfo,
};
use copper_piper::{
	base::{Node, NodeBuilder, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	CopperContext,
};
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, trace};

pub struct AddItem {}

impl NodeBuilder for AddItem {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>> {
		Box::new(Self {})
	}
}

// Inputs: depends on class
// Outputs: None
#[async_trait]
impl<'ctx> Node<'ctx> for AddItem {
	async fn run(
		&self,
		ctx: &CopperContext<'ctx>,
		_this_node: ThisNodeInfo,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
		mut input: BTreeMap<PortName, Option<PipeData>>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
		let mut trans = ctx.item_db_transaction.lock().await;

		//
		// Extract parameters
		//
		let class: ClassInfo = if let Some(value) = params.remove("class") {
			match value {
				NodeParameterValue::Integer(x) => ctx
					.itemdb_client
					.get_class(&mut trans, x.into())
					.await
					.map_err(|e| match e {
						GetClassError::NotFound => RunNodeError::BadParameterOther {
							parameter: "class".into(),
							message: "this class doesn't exist".into(),
						},
						_ => RunNodeError::Other(Arc::new(e)),
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
						.itemdb_client
						.get_dataset(&mut trans, x.into())
						.await
						.map_err(|e| match e {
							GetDatasetError::NotFound => RunNodeError::BadParameterOther {
								parameter: "dataset".into(),
								message: "this dataset doesn't exist".into(),
							},
							_ => RunNodeError::Other(Arc::new(e)),
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

		debug!(message = "Getting attributes");
		let mut attributes: BTreeMap<PortName, (AttributeInfo, Option<AttrData>)> = class
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
					let mut reader = source.build(ctx).await?;

					let mut upload = ctx
						.objectstore_client
						.create_multipart_upload(
							&ctx.objectstore_blob_bucket,
							&new_obj_key,
							reader.mime().clone(),
						)
						.await
						.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

					while let Some(data) = reader.next_fragment().await? {
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

					attr.1 = Some(AttrData::Blob {
						bucket: ctx.objectstore_blob_bucket.clone(),
						key: new_obj_key,
					})
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

					attr.1 = Some(as_attr);
				}

				None => {}
			};
		}

		//
		// Set up and send transaction
		//

		let new_item = ctx
			.itemdb_client
			.add_item(
				&mut trans,
				class.id,
				attributes
					.into_iter()
					.map(|(_, (k, d))| (k.id, d))
					.filter_map(|(k, v)| v.map(|v| (k, v)))
					.collect(),
			)
			.await
			.map_err(|x| RunNodeError::Other(Arc::new(x)))?;

		let mut output = BTreeMap::new();
		output.insert(
			PortName::new("new_item"),
			PipeData::Reference {
				class: class.id,
				item: new_item,
			},
		);

		return Ok(output);
	}
}
