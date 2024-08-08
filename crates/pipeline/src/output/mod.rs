use serde::Deserialize;
use serde_with::serde_as;

use crate::{
	data::{PipelineData, PipelineDataType},
	portspec::PipelinePortSpec,
	syntax::labels::PipelinePortLabel,
};

pub mod storage;

pub trait PipelineOutput {
	// TODO: better errors
	type ErrorKind: Send + Sync;

	fn run(&mut self, data: Vec<&PipelineData>) -> Result<(), Self::ErrorKind>;
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

		class: String,
	},
}

impl PipelineOutputKind {
	pub fn get_inputs(&self) -> PipelinePortSpec {
		match self {
			Self::DataSet { attrs, .. } => PipelinePortSpec::Vec(attrs),
		}
	}

	pub fn get_outputs(&self) -> PipelinePortSpec {
		match self {
			// TODO: output foreign key of added item
			Self::DataSet { .. } => PipelinePortSpec::Static(&[]),
		}
	}
}
