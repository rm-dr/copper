//! A user-provided pipeline specification

use serde::{de::DeserializeOwned, Deserialize};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, HashMap},
	fmt::Debug,
};

use super::ports::NodeOutput;
use crate::{
	api::PipelineData,
	dispatcher::NodeParameterValue,
	labels::{PipelineNodeID, PipelinePortID},
};

/// A description of a node in a pipeline
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct PipelineNodeSpec<DataType: PipelineData> {
	/// What kind of node is this?
	#[serde(rename = "node")]
	pub node_type: SmartString<LazyCompact>,

	/// Parameters for this node
	#[serde(rename = "params")]
	#[serde(default)]
	pub node_params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde(rename = "input")]
	pub inputs: BTreeMap<PipelinePortID, NodeOutput>,

	#[serde(default)]
	/// Nodes that must complete before this node starts
	pub after: Vec<PipelineNodeID>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(in super::super) struct PipelineSpec<DataType: PipelineData> {
	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	pub nodes: HashMap<PipelineNodeID, PipelineNodeSpec<DataType>>,
}
