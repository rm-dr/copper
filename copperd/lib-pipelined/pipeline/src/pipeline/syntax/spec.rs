//! A user-provided pipeline specification

use serde::{de::DeserializeOwned, Deserialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug};

use super::ports::{InputPort, OutputPort};
use crate::{api::PipelineData, dispatcher::NodeParameterValue, labels::PipelineNodeID};

#[derive(Debug, Deserialize, Clone, Copy)]
pub(crate) enum EdgeType {
	Data,
	After,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct NodePosition {
	pub x: i64,
	pub y: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct NodeData<DataType: PipelineData> {
	/// What kind of node is this?
	pub node_type: SmartString<LazyCompact>,

	// Parameters for this node
	#[serde(default)]
	pub params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdgeData {
	/// What kind of edge is this?
	pub edge_type: EdgeType,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct PipelineNodeSpec<DataType: PipelineData> {
	pub position: NodePosition,
	pub data: NodeData<DataType>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineEdgeSpec {
	pub source: OutputPort,
	pub target: InputPort,

	pub data: EdgeData,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub struct PipelineSpec<DataType: PipelineData> {
	/// Nodes in this pipeline
	pub(crate) nodes: BTreeMap<PipelineNodeID, PipelineNodeSpec<DataType>>,

	/// Edges in this pipeline
	pub(crate) edges: BTreeMap<SmartString<LazyCompact>, PipelineEdgeSpec>,
}
