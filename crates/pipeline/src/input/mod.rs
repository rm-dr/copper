use crate::{portspec::PipelinePortSpec, syntax::labels::PipelinePortLabel};
use serde::Deserialize;
use serde_with::serde_as;
use ufo_util::data::{PipelineData, PipelineDataType};

pub mod file;

pub trait PipelineInput {
	type ErrorKind: Send + Sync;

	fn run(self) -> Result<Vec<PipelineData>, Self::ErrorKind>;
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineInputKind {
	File,
	Plain {
		#[serde(rename = "input")]
		#[serde_as(as = "serde_with::Map<_, _>")]
		inputs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},
}

impl PipelineInputKind {
	pub fn get_outputs(&self) -> PipelinePortSpec {
		match self {
			// Order must match
			Self::File => PipelinePortSpec::Static(&[
				("path", PipelineDataType::Text),
				("data", PipelineDataType::Binary),
			]),
			Self::Plain { inputs, .. } => PipelinePortSpec::Vec(inputs),
		}
	}
}
