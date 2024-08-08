use serde::Deserialize;
use std::collections::HashMap;

use super::{nodes::PipelineNodes, PipelineDataType};

#[derive(Debug, Deserialize)]
pub struct Pipeline {
	/// Pipeline parameters
	pipeline: PipelineConfig,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	nodes: HashMap<String, PipelineNodeSpec>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineConfig {
	/// Names and types of pipeline inputs
	#[serde(default)]
	input: HashMap<String, PipelineDataType>,

	/// Names and types of pipeline outputs
	#[serde(default)]
	output: HashMap<String, PipelineDataType>,

	/// Map pipeline outputs to the node outputs that produce them
	#[serde(default)]
	outmap: HashMap<String, PipelineLink>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	node_type: PipelineNodes,

	/// Where this node should read its input from.
	#[serde(default)]
	input: HashMap<String, PipelineLink>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PipelineLink {
	/// Inline static text
	InlineText { text: String },

	/// Get data from another node's output
	Link(String),
}
