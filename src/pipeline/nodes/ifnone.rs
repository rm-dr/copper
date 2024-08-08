use std::collections::HashMap;

use super::PipelineNode;
use crate::pipeline::{
	components::PipelinePortLabel,
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
};

pub struct IfNone {}

impl PipelineNode for IfNone {
	fn get_input(input: &PipelinePortLabel) -> Option<PipelineDataType> {
		match AsRef::as_ref(input) {
			"data" | "ifnone" => Some(PipelineDataType::Text),
			_ => None,
		}
	}

	fn get_output(input: &PipelinePortLabel) -> Option<PipelineDataType> {
		match AsRef::as_ref(input) {
			"out" => Some(PipelineDataType::Text),
			_ => None,
		}
	}

	fn get_inputs() -> impl Iterator<Item = PipelinePortLabel> {
		["data", "ifnone"].iter().map(|x| (*x).into())
	}

	fn get_outputs() -> impl Iterator<Item = PipelinePortLabel> {
		["out"].iter().map(|x| (*x).into())
	}

	fn run(
		mut inputs: HashMap<PipelinePortLabel, Option<PipelineData>>,
	) -> Result<HashMap<PipelinePortLabel, Option<PipelineData>>, PipelineError> {
		let ifnone = inputs.remove(&"ifnone".into()).unwrap();
		let data = inputs.remove(&"data".into()).unwrap();
		return Ok(HashMap::from([(
			"out".into(),
			data.map(Some).unwrap_or(ifnone),
		)]));
	}
}
