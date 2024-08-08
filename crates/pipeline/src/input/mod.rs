use crate::portspec::PipelinePortSpec;
use serde::Deserialize;
use ufo_util::data::{PipelineData, PipelineDataType};

pub mod file;

pub trait PipelineInput {
	type ErrorKind: Send + Sync;

	fn run(self) -> Result<Vec<PipelineData>, Self::ErrorKind>;
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineInputKind {
	File,
}

impl PipelineInputKind {
	pub fn get_outputs(&self) -> PipelinePortSpec {
		match self {
			// Order must match
			Self::File => PipelinePortSpec::Static(&[
				("path", PipelineDataType::Text),
				("data", PipelineDataType::Binary),
			]),
		}
	}
}
