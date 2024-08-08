use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::labels::{PipelineLabel, PipelineNodeLabel};

use super::data::{ApiData, ApiDataStub};

#[derive(Deserialize, Serialize, Debug)]
pub struct AddJobParams {
	pub pipeline: PipelineLabel,
	pub input: Vec<ApiData>,
	pub bound_upload_job: Option<SmartString<LazyCompact>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum AddJobResult {
	Ok, // TODO: return job id
	BadPipeline { pipeline: PipelineLabel },
	InvalidNumberOfArguments { got: usize, expected: usize },
	InvalidInputType { bad_input_idx: usize },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PipelineInfo {
	pub name: PipelineLabel,
	pub nodes: Vec<PipelineNodeLabel>,
	pub input_node: PipelineNodeLabel,
	pub output_node: PipelineNodeLabel,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeInfo {
	pub name: PipelineNodeLabel,

	/// A list of types each of this node's inputs accepts
	pub inputs: Vec<Vec<ApiDataStub>>,
}
