use async_trait::async_trait;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_piper::{
	base::{Node, NodeOutput, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	CopperContext, JobRunResult,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tokio::sync::mpsc;

pub struct Constant {}

#[async_trait]
impl<Itemdb: ItemdbClient> Node<JobRunResult, PipeData, CopperContext<Itemdb>> for Constant {
	async fn run(
		&self,
		_ctx: &CopperContext<Itemdb>,
		this_node: ThisNodeInfo,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
		mut input: BTreeMap<PortName, Option<PipeData>>,
		output: mpsc::Sender<NodeOutput<PipeData>>,
	) -> Result<(), RunNodeError<PipeData>> {
		//
		// Extract parameters
		//
		let value = if let Some(value) = params.remove("value") {
			// Convert parameter into pipeline data
			match value {
				NodeParameterValue::String(value) => PipeData::Text { value },
				NodeParameterValue::Boolean(value) => PipeData::Boolean { value },
				NodeParameterValue::Integer(value) => PipeData::Integer {
					value,
					is_non_negative: false,
				},
				_ => {
					return Err(RunNodeError::BadParameterType {
						parameter: "value".into(),
					})
				}
			}
		} else {
			return Err(RunNodeError::MissingParameter {
				parameter: "value".into(),
			});
		};
		if let Some((param, _)) = params.first_key_value() {
			return Err(RunNodeError::UnexpectedParameter {
				parameter: param.clone(),
			});
		}

		//
		// Extract input
		//
		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}

		//
		// Return the value we were given
		//
		output
			.send(NodeOutput {
				node: this_node,
				port: PortName::new("out"),
				data: Some(value),
			})
			.await?;

		return Ok(());
	}
}
