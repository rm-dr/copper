use crate::syntax::labels::PipelinePortLabel;
use serde::Deserialize;
use std::sync::Arc;
use ufo_util::data::{PipelineData, PipelineDataType};

pub mod file;

pub trait PipelineInput {
	type ErrorKind: Send + Sync;

	fn injest(self) -> Result<Vec<Option<Arc<PipelineData>>>, Self::ErrorKind>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineInputKind {
	File,
}

impl PipelineInputKind {
	pub fn get_outputs(&self) -> Vec<(PipelinePortLabel, PipelineDataType)> {
		match self {
			// Order must match
			Self::File => vec![
				("path".into(), PipelineDataType::Text),
				("data".into(), PipelineDataType::Binary),
			],
		}
	}
}
