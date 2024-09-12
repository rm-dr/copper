use async_trait::async_trait;
use copper_pipelined::{
	base::{Node, NodeParameterValue, PortName, RunNodeError},
	data::PipeData,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct Constant {}

#[async_trait]
impl Node<PipeData, CopperContext> for Constant {
	async fn run(
		&self,
		_ctx: &CopperContext,
		mut params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
		mut input: BTreeMap<PortName, PipeData>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
		//
		// Extract parameters
		//
		let value = if let Some(value) = params.remove("value") {
			match value {
				NodeParameterValue::Data(data) => data.clone(),
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
		let mut out = BTreeMap::new();
		out.insert(PortName::new("out"), value);
		return Ok(out);
	}
}
