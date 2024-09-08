use copper_util::graph::{finalized::FinalizedGraph, graph::Graph};
use pipelined_node_base::base::{
	InitNodeError, NodeDispatcher, NodeParameterValue, PipelineData, PipelineDataStub,
	PipelineJobContext, PipelineNodeID, PipelinePortID, INPUT_NODE_TYPE_NAME,
};
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
		context: &ContextType,
		pipeline_name: &str,
		json: &PipelineJson<DataType>,
	) -> Result<Self, PipelineBuildError<DataType>> {
		debug!(message = "Building pipeline", pipeline_name);

		// Initialize all variables
		let mut graph = Graph::new();
		let mut node_output_name_map_ptp = HashMap::new();
		let mut node_input_name_map_ptp = HashMap::new();
		let mut node_output_name_map_after = HashMap::new();
		let mut node_input_name_map_after = HashMap::new();

		// Add nodes to the graph
		trace!(message = "Making nodes", pipeline_name);
		for (node_id, node_spec) in &json.nodes {
			let n = graph.add_node(NodeSpec {
				id: node_id.clone(),
				node_params: node_spec.data.params.clone(),
				node_type: node_spec.data.node_type.clone(),
			});

			node_output_name_map_ptp.insert(node_id.clone(), n);
			node_input_name_map_ptp.insert(node_id.clone(), n);
			node_output_name_map_after.insert(node_id.clone(), n);
			node_input_name_map_after.insert(node_id.clone(), n);
		}

		// Make sure all "after" edges are valid and create them in the graph.
		trace!(message = "Making `after` edges", pipeline_name);
		for (edge_id, edge_spec) in json
			.edges
			.iter()
			.filter(|(_, v)| matches!(v.data.edge_type, EdgeType::After))
		{
			let source = node_input_name_map_after
				.get(&edge_spec.source.node)
				.ok_or(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.source.node.clone(),
				})?;
			let target = node_input_name_map_after
				.get(&edge_spec.target.node)
				.ok_or(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.target.node.clone(),
				})?;

			graph.add_edge(source.clone(), target.clone(), EdgeSpec::After);
		}

		// Make sure all "data" edges are valid and create them in the graph.
		trace!(message = "Making `data` edges", pipeline_name);
		for (edge_id, edge_spec) in json
			.edges
			.iter()
			.filter(|(_, v)| matches!(v.data.edge_type, EdgeType::Data))
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

			let source_node_info = dispatcher
				.node_info(
					context,
					&source_node.data.node_type,
					&source_node.data.params,
					edge_spec.source.node.id(),
				)?
				.unwrap();

			let target_node_info = dispatcher
				.node_info(
					context,
					&target_node.data.node_type,
					&target_node.data.params,
					edge_spec.target.node.id(),
				)?
				.unwrap();

			// Make sure types are compatible
			{
				let source_type = *source_node_info
					.outputs()
					.get(&edge_spec.source.port)
					.ok_or(PipelineBuildError::NoNode {
						edge_id: edge_id.clone(),
						invalid_node_id: edge_spec.source.node.clone(),
					})?;

				let target_type = *target_node_info
					.outputs()
					.get(&edge_spec.target.port)
					.ok_or(PipelineBuildError::NoNode {
						edge_id: edge_id.clone(),
						invalid_node_id: edge_spec.target.node.clone(),
					})?;

				if !source_type.is_subset_of(&target_type) {
					return Err(PipelineBuildError::TypeMismatch {
						edge_id: edge_id.clone(),
						source_type,
						target_type,
					});
				}
			}

			if !source_node_info
				.inputs()
				.contains_key(&edge_spec.source.port)
			{
				return Err(PipelineBuildError::NoSuchOutputPort {
					edge_id: edge_id.clone(),
					node: edge_spec.source.node.clone(),
					invalid_port: edge_spec.source.port.clone(),
				});
			};

			if !target_node_info
				.inputs()
				.contains_key(&edge_spec.target.port)
			{
				return Err(PipelineBuildError::NoSuchOutputPort {
					edge_id: edge_id.clone(),
					node: edge_spec.target.node.clone(),
					invalid_port: edge_spec.target.port.clone(),
				});
			};

			let source_node_idx = *node_output_name_map_ptp
				.get(&edge_spec.source.node)
				.unwrap();

			let target_node_idx = *node_input_name_map_ptp.get(&edge_spec.target.node).unwrap();

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

		return Ok(Self {
			_pa: PhantomData {},
			_pb: PhantomData {},
			name: pipeline_name.into(),
			graph: graph.finalize(),
		});
	}

	/// Iterate over all nodes in this pipeline
	pub fn iter_node_ids(&self) -> impl Iterator<Item = &PipelineNodeID> {
		self.graph.iter_nodes().map(|n| &n.id)
	}

	/// Get this pipeline's name
	pub fn get_name(&self) -> &str {
		&self.name
	}

	pub fn get_node(&self, node_id: &PipelineNodeID) -> Option<&NodeSpec<DataType>> {
		self.graph.iter_nodes().find(|n| n.id == *node_id)
	}

	pub fn input_nodes(&self) -> Vec<(PipelineNodeID, <DataType as PipelineData>::DataStubType)> {
		self.graph
			.iter_nodes()
			.filter(|n| n.node_type == INPUT_NODE_TYPE_NAME)
			.map(|n| {
				(
					n.id.clone(),
					match n.node_params.get("data_type") {
						Some(NodeParameterValue::DataType(x)) => *x,
						_ => unreachable!(),
					},
				)
			})
			.collect()
	}
}

//
// MARK: Nodes & Edges
//

#[derive(Debug)]
pub struct NodeSpec<DataType: PipelineData> {
	/// The node's id
	pub id: PipelineNodeID,

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
	PortToPort((PipelinePortID, PipelinePortID)),

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
	pub fn source_port(&self) -> Option<PipelinePortID> {
		match self {
			Self::PortToPort((s, _)) => Some(s.clone()),
			Self::After => None,
		}
	}

	/// Get the port this edge ends at
	pub fn target_port(&self) -> Option<PipelinePortID> {
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
pub enum PipelineBuildError<DataType: PipelineData> {
	/// An edge references a node, but it doesn't exist
	NoNode {
		/// The edge that references an invalid node
		edge_id: SmartString<LazyCompact>,

		/// The node id that doesn't exist
		invalid_node_id: PipelineNodeID,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		/// The offending edge
		edge_id: SmartString<LazyCompact>,

		/// The source type
		source_type: <DataType as PipelineData>::DataStubType,

		/// The incompatible target type
		target_type: <DataType as PipelineData>::DataStubType,
	},

	/// This pipeline has a cycle and is thus invalid
	HasCycle,

	/// `node` has no output port named `input`.
	/// This is triggered when we specify an input that doesn't exist.
	NoSuchOutputPort {
		/// The responsible edge
		edge_id: SmartString<LazyCompact>,
		/// The node we tried to reference
		node: PipelineNodeID,
		/// The port that doesn't exist
		invalid_port: PipelinePortID,
	},

	/// `node` has no input port named `port`.
	/// This is triggered when we specify an input that doesn't exist.
	NoSuchInputPort {
		/// The responsible edge
		edge_id: SmartString<LazyCompact>,
		/// The node we tried to reference
		node: PipelineNodeID,
		/// The port name that doesn't exist
		invalid_port: PipelinePortID,
	},

	/// We tried to create a node with an unrecognized type
	InvalidNodeType {
		/// The node that was invalid
		node: PipelineNodeID,

		///The invalid type
		bad_type: SmartString<LazyCompact>,
	},

	/// We encountered an [`InitNodeError`] while building a pipeline
	InitNodeError {
		/// The error we encountered
		error: InitNodeError,
	},
}

impl<DataType: PipelineData> Display for PipelineBuildError<DataType> {
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

			Self::TypeMismatch {
				edge_id,
				source_type,
				target_type,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` connects incompatible types `{source_type:?}` and `{target_type:?}`"
				)
			}

			Self::HasCycle => {
				writeln!(f, "this pipeline has a cycle")
			}

			Self::NoSuchInputPort {
				edge_id,
				node,
				invalid_port,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references invalid input port `{invalid_port}` on node `{node}`"
				)
			}

			Self::NoSuchOutputPort {
				edge_id,
				node,
				invalid_port,
			} => {
				writeln!(
					f,
					"edge `{edge_id}` references invalid output port `{invalid_port}` on node `{node}`"
				)
			}

			Self::InvalidNodeType { node, bad_type } => {
				writeln!(f, "node `{node}` has invalid type `{bad_type}`")
			}

			Self::InitNodeError { .. } => {
				writeln!(f, "could not initialize node")
			}
		}
	}
}

impl<DataType: PipelineData> Error for PipelineBuildError<DataType> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::InitNodeError { error } => Some(error),
			_ => None,
		}
	}
}

impl<DataType: PipelineData> From<InitNodeError> for PipelineBuildError<DataType> {
	fn from(error: InitNodeError) -> Self {
		Self::InitNodeError { error }
	}
}
