use copper_pipelined::base::{NodeId, NodeParameterValue, PipelineData, PortName};
use serde::{de::DeserializeOwned, Deserialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug};
use utoipa::ToSchema;

/// A pipeline specification, directly deserialized from JSON.
/// This is the first step in our pipeline processing workflow.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub struct PipelineJson<DataType: PipelineData> {
	/// Nodes in this pipeline
	#[schema(value_type = BTreeMap<String, NodeJson<DataType>>)]
	pub(crate) nodes: BTreeMap<NodeId, NodeJson<DataType>>,

	/// Edges in this pipeline
	#[schema(value_type = BTreeMap<String, EdgeJson>)]
	pub(crate) edges: BTreeMap<SmartString<LazyCompact>, EdgeJson>,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
#[serde(bound = "DataType: DeserializeOwned")]
pub(crate) struct NodeJson<DataType: PipelineData> {
	/// What kind of node is this?
	#[schema(value_type = String)]
	pub node_type: SmartString<LazyCompact>,

	// Parameters for this node
	#[serde(default)]
	#[schema(value_type = BTreeMap<String, NodeParameterValue<DataType>>)]
	pub params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdgeJson {
	pub source: OutputPort,
	pub target: InputPort,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct OutputPort {
	/// The node that provides this output
	#[schema(value_type = String)]
	pub node: NodeId,

	/// The output's name
	#[schema(value_type = String)]
	pub port: PortName,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct InputPort {
	/// The node that provides this input
	#[schema(value_type = String)]
	pub node: NodeId,

	/// The port's name
	#[schema(value_type = String)]
	pub port: PortName,
}