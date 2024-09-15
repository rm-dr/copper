use copper_pipelined::base::{NodeId, NodeParameterValue, PortName};
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug};
use utoipa::ToSchema;

/// A pipeline specification, directly deserialized from JSON.
/// This is the first step in our pipeline processing workflow.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineJson {
	/// Nodes in this pipeline
	#[schema(value_type = BTreeMap<String, NodeJson>)]
	pub(crate) nodes: BTreeMap<NodeId, NodeJson>,

	/// Edges in this pipeline
	#[schema(value_type = BTreeMap<String, EdgeJson>)]
	pub(crate) edges: BTreeMap<SmartString<LazyCompact>, EdgeJson>,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct NodeJson {
	/// What kind of node is this?
	#[schema(value_type = String)]
	pub node_type: SmartString<LazyCompact>,

	// Parameters for this node
	#[serde(default)]
	#[schema(value_type = BTreeMap<String, NodeParameterValue>)]
	pub params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
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
