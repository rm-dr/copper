use crate::base::{NodeId, NodeParameterValue, PortName};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug};
use utoipa::ToSchema;

/// A pipeline specification, directly deserialized from JSON.
/// This is the first step in our pipeline processing workflow.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineJson {
	/// Nodes in this pipeline
	#[schema(value_type = BTreeMap<String, NodeJson>)]
	pub nodes: BTreeMap<NodeId, NodeJson>,

	/// Edges in this pipeline
	#[schema(value_type = BTreeMap<String, EdgeJson>)]
	pub edges: BTreeMap<SmartString<LazyCompact>, EdgeJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NodeJson {
	/// What kind of node is this?
	#[schema(value_type = String)]
	pub node_type: SmartString<LazyCompact>,

	// Parameters for this node
	#[serde(default)]
	#[schema(value_type = BTreeMap<String, NodeParameterValue>)]
	pub params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EdgeJson {
	pub source: OutputPort,
	pub target: InputPort,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputPort {
	/// The node that provides this output
	#[schema(value_type = String)]
	pub node: NodeId,

	/// The output's name
	#[schema(value_type = String)]
	pub port: PortName,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct InputPort {
	/// The node that provides this input
	#[schema(value_type = String)]
	pub node: NodeId,

	/// The port's name
	#[schema(value_type = String)]
	pub port: PortName,
}
