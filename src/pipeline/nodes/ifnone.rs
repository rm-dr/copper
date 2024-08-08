use std::collections::HashMap;

use smartstring::{LazyCompact, SmartString};

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
		mut inputs: HashMap<SmartString<LazyCompact>, PipelineData>,
	) -> Result<HashMap<SmartString<LazyCompact>, PipelineData>, PipelineError> {
		let data = inputs
			.remove("data")
			.unwrap_or(inputs.remove("ifnone").unwrap());
		return Ok(HashMap::from([("out".into(), data)]));
	}
}
