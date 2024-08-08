use std::collections::HashMap;

use super::PipelineNode;
use crate::pipeline::{
	components::PipelinePort,
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
};

pub struct IfNone {}

impl PipelineNode for IfNone {
	fn get_input(input: &PipelinePort) -> Option<PipelineDataType> {
		match AsRef::as_ref(input) {
			"data" | "ifnone" => Some(PipelineDataType::Text),
			_ => None,
		}
	}

	fn get_output(input: &PipelinePort) -> Option<PipelineDataType> {
		match AsRef::as_ref(input) {
			"out" => Some(PipelineDataType::Text),
			_ => None,
		}
	}

	fn get_inputs() -> impl Iterator<Item = PipelinePort> {
		["data", "ifnone"].iter().map(|x| (*x).into())
	}

	fn get_outputs() -> impl Iterator<Item = PipelinePort> {
		["out"].iter().map(|x| (*x).into())
	}

	fn run<F>(
		get_input: F,
	) -> Result<HashMap<PipelinePort, Option<PipelineData>>, PipelineError>
	where
		F: Fn(&PipelinePort) -> Option<PipelineData>,
	{
		// TODO: don't clone, link (replace Option<>)
		let ifnone = get_input(&"ifnone".into()).unwrap();
		let data = get_input(&"data".into());
		return Ok(HashMap::from([(
			"out".into(),
			//Some(data.cloned().unwrap_or(ifnone.clone())),
			Some(data.unwrap_or(ifnone)),
		)]));
	}
}
