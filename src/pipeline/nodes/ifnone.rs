use std::collections::HashMap;

use super::PipelineNode;
use crate::pipeline::{PipelineData, PipelineDataType, PipelineError};

pub struct IfNone {}

impl PipelineNode for IfNone {
	fn get_inputs() -> &'static [(&'static str, PipelineDataType)] {
		&[
			("data", PipelineDataType::Text),
			("ifnone", PipelineDataType::Text),
		]
	}

	fn get_outputs() -> &'static [(&'static str, PipelineDataType)] {
		&[("out", PipelineDataType::Text)]
	}

	fn run(
		mut inputs: HashMap<String, PipelineData>,
	) -> Result<HashMap<String, PipelineData>, PipelineError> {
		let data = inputs
			.remove("data")
			.unwrap_or(inputs.remove("ifnone").unwrap());
		return Ok(HashMap::from([("out".to_string(), data)]));
	}
}
