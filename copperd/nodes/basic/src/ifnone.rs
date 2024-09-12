use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeParameterValue, PipelineData, PortName, RunNodeError},
	data::PipeData,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct IfNone {}

// Inputs:
// - "data", <T>
// - "ifnone", <T>
// Outputs:
// - "out", <T>
#[async_trait]
impl Node<PipeData, CopperContext> for IfNone {
	async fn run(
		&self,
		_ctx: &CopperContext,
		params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, PipeData>,
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
		let ifnone = input.remove(&PortName::new("ifnone"));
		if ifnone.is_none() {
			return Err(RunNodeError::MissingInput {
				port: PortName::new("ifnone"),
			});
		}
		let ifnone = ifnone.unwrap();
		let data = input.remove(&PortName::new("data"));

		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}
		if data.is_some() && ifnone.as_stub() != data.as_ref().unwrap().as_stub() {
			return Err(RunNodeError::BadInputType {
				port: PortName::new("ifnone"),
			});
		}

		//
		// Return the correct value
		//
		let mut out = BTreeMap::new();
		out.insert(PortName::new("out"), data.unwrap_or(ifnone));
		return Ok(out);
	}
}
