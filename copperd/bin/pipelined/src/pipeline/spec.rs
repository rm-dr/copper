use copper_pipelined::base::{
	NodeDispatcher, NodeId, NodeParameterValue, PipelineData, PipelineJobContext, PortName,
};
use copper_util::graph::{finalized::FinalizedGraph, graph::Graph};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, HashMap},
	error::Error,
	fmt::{Debug, Display},
	marker::PhantomData,
};
use tracing::{debug, trace};

use crate::pipeline::json::EdgeType;

use super::json::PipelineJson;

//
// MARK: PipelineSpec
//

/// A pipeline specification built from [`PipelineJson`].
///
/// This is the second step in our pipeline processing workflow.
/// Any [`PipelineJson`] that builds into a PipelineSpec successfully
/// should be runnable (but may encounter run-time errors)
#[derive(Debug)]
pub struct PipelineSpec<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	pub(crate) _pa: PhantomData<DataType>,
	pub(crate) _pb: PhantomData<ContextType>,

	/// This pipeline's name.
	pub(crate) name: SmartString<LazyCompact>,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<NodeSpec<DataType>, EdgeSpec>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	PipelineSpec<DataType, ContextType>
{
	/// Load a pipeline from a TOML string
	pub fn build(
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		pipeline_name: &str,
		json: &PipelineJson<DataType>,
	) -> Result<Self, PipelineBuildError> {
		debug!(message = "Building pipeline", pipeline_name);

		// The graph that stores this pipeline
		let mut graph = Graph::new();
		// Maps node ids (from JSON) to node indices in `graph`
		let mut node_id_map = HashMap::new();

		// Create all nodes in the graph
		trace!(message = "Making nodes", pipeline_name);
		for (node_id, node_spec) in &json.nodes {
			let n = graph.add_node(NodeSpec {
				id: node_id.clone(),
				node_params: node_spec.params.clone(),
				node_type: node_spec.node_type.clone(),
			});

			node_id_map.insert(node_id.clone(), n);
		}

		// Make sure all "after" edges are valid and create them in the graph.
		trace!(message = "Making `after` edges", pipeline_name);
		for (edge_id, edge_spec) in json
			.edges
			.iter()
			.filter(|(_, v)| matches!(v.edge_type, EdgeType::After))
		{
			let source =
				node_id_map
					.get(&edge_spec.source.node)
					.ok_or(PipelineBuildError::NoNode {
						edge_id: edge_id.clone(),
						invalid_node_id: edge_spec.source.node.clone(),
					})?;
			let target =
				node_id_map
					.get(&edge_spec.target.node)
					.ok_or(PipelineBuildError::NoNode {
						edge_id: edge_id.clone(),
						invalid_node_id: edge_spec.target.node.clone(),
					})?;

			graph.add_edge(source.clone(), target.clone(), EdgeSpec::After);
		}

		// Make sure all "data" edges are valid and create them in the graph.
		//
		// We do not check if ports exist & have matching types here,
		// since not all nodes know their ports at build time.
		trace!(message = "Making `data` edges", pipeline_name);
		for (edge_id, edge_spec) in json
			.edges
			.iter()
			.filter(|(_, v)| matches!(v.edge_type, EdgeType::Data))
		{
			let source_node =
				json.nodes
					.get(&edge_spec.source.node)
					.ok_or(PipelineBuildError::NoNode {
						edge_id: edge_id.clone(),
						invalid_node_id: edge_spec.source.node.clone(),
					})?;
			let target_node =
				json.nodes
					.get(&edge_spec.target.node)
					.ok_or(PipelineBuildError::NoNode {
						edge_id: edge_id.clone(),
						invalid_node_id: edge_spec.target.node.clone(),
					})?;

			if !dispatcher.has_node(&source_node.node_type) {
				return Err(PipelineBuildError::InvalidNodeType {
					bad_type: source_node.node_type.clone(),
				});
			}

			if !dispatcher.has_node(&target_node.node_type) {
				return Err(PipelineBuildError::InvalidNodeType {
					bad_type: target_node.node_type.clone(),
				});
			}

			// These should never fail
			let source_node_idx = *node_id_map.get(&edge_spec.source.node).unwrap();
			let target_node_idx = *node_id_map.get(&edge_spec.target.node).unwrap();

			// Create the edge
			graph.add_edge(
				source_node_idx,
				target_node_idx,
				EdgeSpec::PortToPort((
					edge_spec.source.port.clone(),
					edge_spec.target.port.clone(),
				)),
			);
		}

		trace!(message = "Looking for cycles", pipeline_name);
		// Make sure our graph doesn't have any cycles
		if graph.has_cycle() {
			return Err(PipelineBuildError::HasCycle);
		}

		trace!(message = "Pipeline is ready", pipeline_name);
		return Ok(Self {
			_pa: PhantomData {},
			_pb: PhantomData {},
			name: pipeline_name.into(),
			graph: graph.finalize(),
		});
	}

	/// Iterate over all nodes in this pipeline
	pub fn iter_node_ids(&self) -> impl Iterator<Item = &NodeId> {
		self.graph.iter_nodes().map(|n| &n.id)
	}

	/// Get this pipeline's name
	pub fn get_name(&self) -> &str {
		&self.name
	}
}

//
// MARK: Nodes & Edges
//

#[derive(Debug)]
pub struct NodeSpec<DataType: PipelineData> {
	/// The node's id
	pub id: NodeId,

	/// This node's type
	pub node_type: SmartString<LazyCompact>,

	/// This node's parameters
	pub node_params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
}

#[derive(Debug, Clone)]
pub enum EdgeSpec {
	/// A edge from an output port to an input port.
	/// PTP edges carry data between nodes.
	///
	/// Contents are (from_port, to_port)
	PortToPort((PortName, PortName)),

	/// An edge from a node to a node, specifying
	/// that the second *must* wait for the first.
	After,
}

impl EdgeSpec {
	/// Is this a `Self::PortToPort`?
	pub fn is_ptp(&self) -> bool {
		matches!(self, Self::PortToPort(_))
	}

	/// Is this a `Self::After`?
	pub fn is_after(&self) -> bool {
		matches!(self, Self::After)
	}

	/// Get the port this edge starts at
	pub fn source_port(&self) -> Option<PortName> {
		match self {
			Self::PortToPort((s, _)) => Some(s.clone()),
			Self::After => None,
		}
	}

	/// Get the port this edge ends at
	pub fn target_port(&self) -> Option<PortName> {
		match self {
			Self::PortToPort((_, t)) => Some(t.clone()),
			Self::After => None,
		}
	}
}

//
// MARK: Errors
//

/// An error we encounter when a pipeline spec is invalid
#[derive(Debug)]
pub enum PipelineBuildError {
	/// An edge references a node, but it doesn't exist
	NoNode {
		/// The edge that references an invalid node
		edge_id: SmartString<LazyCompact>,

		/// The node id that doesn't exist
		invalid_node_id: NodeId,
	},

	/// This pipeline has a cycle and is thus invalid
	HasCycle,

	/// We tried to create a node with an unrecognized type
	InvalidNodeType {
		/// The invalid type
		bad_type: SmartString<LazyCompact>,
	},
}

impl Display for PipelineBuildError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NoNode {
				edge_id,
				invalid_node_id,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references a node `{invalid_node_id}` that doesn't exist"
				)
			}

			Self::HasCycle => {
				writeln!(f, "this pipeline has a cycle")
			}

			Self::InvalidNodeType { bad_type } => {
				writeln!(f, "unknown node type `{bad_type}`")
			}
		}
	}
}

impl Error for PipelineBuildError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			_ => None,
		}
	}
}
