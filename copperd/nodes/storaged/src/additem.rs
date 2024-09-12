use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeOutput, NodeParameterValue, PipelineData, PortName, RunNodeError},
	data::{PipeData, PipeDataStub},
	CopperContext,
};
use copper_storaged::{AttrData, AttributeInfo, ClassId, Transaction, TransactionAction};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{debug, trace};

pub struct AddItem {}

// Inputs: depends on class
// Outputs: None
#[async_trait]
impl Node<PipeData, CopperContext> for AddItem {
	async fn run(
		&self,
		ctx: &CopperContext,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, NodeOutput<PipeData>>,
	) -> Result<BTreeMap<PortName, NodeOutput<PipeData>>, RunNodeError> {
		//
		// Extract parameters
		//
		let class: ClassId = if let Some(value) = params.remove("class") {
			match value {
				NodeParameterValue::Integer(x) => x.into(),
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
		let mut attributes: BTreeMap<PortName, (AttributeInfo, Option<AttrData>)> = ctx
			.storaged_client
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

			match data.get_value().await? {
				Some(PipeData::Blob { .. }) => {
					unimplemented!()
				}

				Some(x) => {
					let attr = attributes.get_mut(&port).unwrap();

					// Check data type
					match x.as_stub() {
						PipeDataStub::Plain { data_type } => {
							if data_type != attr.0.data_type {
								return Err(RunNodeError::BadInputType { port });
							}
						}
					}

					attr.1 = Some(x.try_into().unwrap())
				}

				None => {}
			};
		}

		//
		// Set up and send transaction
		//
		let transaction = Transaction {
			actions: vec![TransactionAction::AddItem {
				to_class: class,
				attributes: attributes
					.into_iter()
					.map(|(_, (k, d))| {
						(
							k.id,
							d.unwrap_or(AttrData::None {
								data_type: k.data_type,
							}),
						)
					})
					.collect(),
			}],
		};

		debug!(message = "Sending transaction", ?transaction);
		ctx.storaged_client
			.apply_transaction(transaction)
			.await
			.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

		return Ok(BTreeMap::new());
	}
}
