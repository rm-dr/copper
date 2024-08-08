use crate::syntax::labels::PipelinePortLabel;
use serde::Deserialize;
use ufo_util::data::{PipelineData, PipelineDataType};

pub mod storage;

pub trait PipelineOutput {
	type ErrorKind: Send + Sync;

	fn export(&mut self, data: Vec<Option<&PipelineData>>) -> Result<(), Self::ErrorKind>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineOutputKind {
	DataSet {
		#[serde(rename = "class")]
		class_name: String,
	},
}

impl PipelineOutputKind {
	pub fn get_inputs(&self) -> Vec<(PipelinePortLabel, PipelineDataType)> {
		match self {
			// Order must match
			Self::DataSet { .. } => vec![
				("artist".into(), PipelineDataType::Text),
				("album".into(), PipelineDataType::Text),
			],
		}
	}
}
