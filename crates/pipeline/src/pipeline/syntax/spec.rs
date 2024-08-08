//! A user-provided pipeline specification

use serde::{de::DeserializeOwned, Deserialize};
use serde_with::{self, serde_as};
use std::{collections::HashMap, fmt::Debug};

use super::ports::NodeOutput;
use crate::{
	api::PipelineNodeStub,
	labels::{PipelineNodeID, PipelinePortID},
};

/// A description of a node in a pipeline
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "NodeStubType: DeserializeOwned")]
pub(crate) struct PipelineNodeSpec<NodeStubType: PipelineNodeStub> {
	/// What kind of node is this?
	#[serde(rename = "node")]
	pub node_type: NodeStubType,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde(rename = "input")]
	#[serde_as(as = "serde_with::Map<_, _>")]
	pub inputs: Vec<(PipelinePortID, NodeOutput<NodeStubType>)>,

	#[serde(default)]
	/// Nodes that must complete before this node starts
	pub after: Vec<PipelineNodeID>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "NodeStubType: DeserializeOwned")]
pub(in super::super) struct PipelineSpec<NodeStubType: PipelineNodeStub> {
	/// This pipeline's input node.
	/// Note that this doesn't provide an `inputs` array.
	/// That is wired up by the code that runs this pipeline.
	pub input: NodeStubType,

	/// This pipeline's output node
	pub output: PipelineNodeSpec<NodeStubType>,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	pub nodes: HashMap<PipelineNodeID, PipelineNodeSpec<NodeStubType>>,
}
