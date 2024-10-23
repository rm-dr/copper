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

pub struct IfNone {}

// Inputs:
// - "data", <T>
// - "ifnone", <T>
// Outputs:
// - "out", <T>
#[async_trait]
impl<Itemdb: ItemdbClient> Node<JobRunResult, PipeData, CopperContext<Itemdb>> for IfNone {
	async fn run(
		&self,
		_ctx: &CopperContext<Itemdb>,
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
		// Extract input
		//

		// Note that we do not have enough information to catch type errors here.
		// We cannot check if the type of `data` matches the type of `ifnone`.
		// This may not be a problem (UI prevents this, and you deserve confusion if
		// you're hand-crafting json), but it would be nice to catch this anyway.
		// TODO: possible solutions are static type analysis (in build()) or
		// a typed `None` pipeline data container.

		// We need data right away, so await it now
		let data = match input.remove(&PortName::new("data")) {
			Some(x) => x,
			None => {
				return Err(RunNodeError::MissingInput {
					port: PortName::new("data"),
				});
			}
		};

		// Don't await `ifnone` yet, we shouldn't need to wait for it
		// unless `data` is None.
		let ifnone = match input.remove(&PortName::new("ifnone")) {
			Some(x) => x,
			None => {
				return Err(RunNodeError::MissingInput {
					port: PortName::new("ifnone"),
				});
			}
		};

		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}

		if let Some(data) = data {
			output
				.send(NodeOutput {
					node: this_node,
					port: PortName::new("out"),
					data: Some(data),
				})
				.await?;
			return Ok(());
		};

		output
			.send(NodeOutput {
				node: this_node,
				port: PortName::new("out"),
				data: ifnone,
			})
			.await?;
		return Ok(());
	}
}
