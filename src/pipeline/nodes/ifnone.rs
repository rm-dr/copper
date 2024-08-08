use std::collections::HashMap;

use smartstring::{LazyCompact, SmartString};

use super::PipelineNode;
use crate::pipeline::{
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
};

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
		mut inputs: HashMap<SmartString<LazyCompact>, Option<PipelineData>>,
	) -> Result<HashMap<SmartString<LazyCompact>, Option<PipelineData>>, PipelineError> {
		let ifnone = inputs.remove("ifnone").unwrap();
		let data = inputs.remove("data").unwrap();
		return Ok(HashMap::from([(
			"out".into(),
			data.map(Some).unwrap_or(ifnone),
		)]));
	}
}
