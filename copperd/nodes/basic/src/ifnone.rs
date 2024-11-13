use async_trait::async_trait;
use copper_piper::{
	base::{Node, NodeBuilder, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct IfNone {}

impl NodeBuilder for IfNone {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>> {
		Box::new(Self {})
	}
}

// Inputs:
// - "data", <T>
// - "ifnone", <T>
// Outputs:
// - "out", <T>
#[async_trait]
impl<'ctx> Node<'ctx> for IfNone {
	async fn run(
		&self,
		_ctx: &CopperContext<'ctx>,
		_this_node: ThisNodeInfo,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
		mut input: BTreeMap<PortName, Option<PipeData>>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
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

		let mut output = BTreeMap::new();

		if let Some(data) = data {
			output.insert(PortName::new("out"), data);
		} else if let Some(ifnone) = ifnone {
			output.insert(PortName::new("out"), ifnone);
		};

		return Ok(output);
	}
}
