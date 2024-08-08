//! A user-provided pipeline specification

use serde::{de::DeserializeOwned, Deserialize};
use serde_with::{self, serde_as};
use std::{collections::HashMap, fmt::Debug};

use super::{internalnode::InternalNodeStub, ports::NodeOutput};
use crate::{
	api::PipelineNodeStub,
	labels::{PipelineNodeLabel, PipelinePortLabel},
};

/// A description of a node in a pipeline
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(crate) struct PipelineNodeSpec<StubType: PipelineNodeStub> {
	/// What kind of node is this?
	#[serde(rename = "node")]
	pub node_type: InternalNodeStub<StubType>,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde(rename = "input")]
	#[serde_as(as = "serde_with::Map<_, _>")]
	pub inputs: Vec<(PipelinePortLabel, NodeOutput<StubType>)>,

	#[serde(default)]
	/// Nodes that must complete before this node starts
	pub after: Vec<PipelineNodeLabel>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(crate) struct PipelineSpec<StubType: PipelineNodeStub> {
	/// This pipeline's input node.
	/// Note that this doesn't provide an `inputs` array.
	/// that is wired up by code that runs this pipeline.
	pub input: InternalNodeStub<StubType>,

	/// This pipeline's output node
	pub output: PipelineNodeSpec<StubType>,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	pub nodes: HashMap<PipelineNodeLabel, PipelineNodeSpec<StubType>>,
}

// TODO: warnings (disconnected input)
// TODO: check for unused nodes
