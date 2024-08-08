//! A user-provided pipeline specification

use serde::{de::DeserializeOwned, Deserialize};
use serde_with::{self, serde_as};
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
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct PipelineNodeSpec<DataType: PipelineData> {
	/// What kind of node is this?
	#[serde(rename = "node")]
	pub node_type: SmartString<LazyCompact>,

	/// Parameters for this node
	#[serde(rename = "params")]
	pub node_params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde(rename = "input")]
	#[serde_as(as = "serde_with::Map<_, _>")]
	pub inputs: Vec<(PipelinePortID, NodeOutput)>,

	#[serde(default)]
	/// Nodes that must complete before this node starts
	pub after: Vec<PipelineNodeID>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(in super::super) struct PipelineSpec<DataType: PipelineData> {
	/// The type of input this pipeline takes
	#[serde(default)]
	pub input: HashMap<SmartString<LazyCompact>, <DataType as PipelineData>::DataStubType>,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	pub nodes: HashMap<PipelineNodeID, PipelineNodeSpec<DataType>>,
}
