use copper_pipelined::base::{NodeId, NodeParameterValue, PipelineData, PortName};
use serde::{de::DeserializeOwned, Deserialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug};

/// A pipeline specification, directly deserialized from JSON.
/// This is the first step in our pipeline processing workflow.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub struct PipelineJson<DataType: PipelineData> {
	/// Nodes in this pipeline
	pub(crate) nodes: BTreeMap<NodeId, NodeJson<DataType>>,

	/// Edges in this pipeline
	pub(crate) edges: BTreeMap<SmartString<LazyCompact>, EdgeJson>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct NodeJson<DataType: PipelineData> {
	pub data: NodeJsonData<DataType>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct NodeJsonData<DataType: PipelineData> {
	/// What kind of node is this?
	pub node_type: SmartString<LazyCompact>,

	// Parameters for this node
	#[serde(default)]
	pub params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdgeJson {
	pub source: OutputPort,
	pub target: InputPort,
	pub data: EdgeJsonData,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdgeJsonData {
	/// What kind of edge is this?
	pub edge_type: EdgeType,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub(crate) enum EdgeType {
	Data,
	After,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OutputPort {
	/// The node that provides this output
	pub node: NodeId,

	/// The output's name
	pub port: PortName,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputPort {
	/// The node that provides this input
	pub node: NodeId,

	/// The port's name
	pub port: PortName,
}
