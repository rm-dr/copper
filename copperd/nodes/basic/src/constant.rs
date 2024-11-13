use async_trait::async_trait;
use copper_piper::{
	base::{Node, NodeBuilder, NodeParameterValue, PortName, RunNodeError, ThisNodeInfo},
	data::PipeData,
	helpers::NodeParameters,
	CopperContext,
};
use std::collections::BTreeMap;

pub struct Constant {}

impl NodeBuilder for Constant {
	fn build<'ctx>(&self) -> Box<dyn Node<'ctx>> {
		Box::new(Self {})
	}
}

#[async_trait]
impl<'ctx> Node<'ctx> for Constant {
	async fn run(
		&self,
		_ctx: &CopperContext<'ctx>,
		_this_node: ThisNodeInfo,
		mut params: NodeParameters,
		mut input: BTreeMap<PortName, Option<PipeData>>,
	) -> Result<BTreeMap<PortName, PipeData>, RunNodeError> {
		//
		// Extract parameters
		//
		let value = {
			let val = params.pop_val("value")?;

			match val {
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
		};

		params.err_if_not_empty()?;

		//
		// Extract input
		//
		if let Some((port, _)) = input.pop_first() {
			return Err(RunNodeError::UnrecognizedInput { port });
		}

		//
		// Return the value we were given
		//

		let mut output = BTreeMap::new();
		output.insert(PortName::new("out"), value);
		return Ok(output);
	}
}
