use crate::syntax::labels::PipelinePortLabel;
use serde::Deserialize;
use serde_with::serde_as;
use ufo_util::data::{PipelineData, PipelineDataType};

pub mod storage;

pub trait PipelineOutput {
	// TODO: better errors
	type ErrorKind: Send + Sync;

	fn export(&mut self, data: Vec<Option<&PipelineData>>) -> Result<(), Self::ErrorKind>;
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum PipelineOutputKind {
	DataSet {
		#[serde(rename = "attr")]
		#[serde_as(as = "serde_with::Map<_, _>")]
		attrs: Vec<(PipelinePortLabel, PipelineDataType)>,
	},
}

impl PipelineOutputKind {
	pub fn get_inputs(&self) -> Vec<(PipelinePortLabel, PipelineDataType)> {
		match self {
			// Order must match
			Self::DataSet { attrs: attr, .. } => attr.clone(),
		}
	}
}
