use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use std::fmt::Debug;

use super::{
	finalized::FinalizedGraph,
	util::{GraphEdgeIdx, GraphNodeIdx},
};

/// A directed graph with parallel edges.
/// Fast writes are not a goal (within reason).
///
/// [`Graph`]s are designed to be created once,
/// (possibly mutated, if creation requires multiple stages),
/// and only read afterwards.
#[derive(Debug, Clone)]
pub struct Graph<NodeType, EdgeType>
where
	NodeType: Debug,
	EdgeType: Debug,
{
	/// Array of nodes in this graph
	pub(super) nodes: Vec<NodeType>,

	/// Array of edges in this graph
	pub(super) edges: Vec<(GraphNodeIdx, GraphNodeIdx, EdgeType)>,
}

impl<NodeType, EdgeType> Graph<NodeType, EdgeType>
where
	NodeType: Debug,
	EdgeType: Debug,
{
	/// Create an empty graph
	pub fn new() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
		}
	}

	/// Convert this graph to an immutable structure with fast reads.
	pub fn finalize(self) -> FinalizedGraph<NodeType, EdgeType> {
		let mut edge_map_in = (0..self.nodes.len())
			.map(|_| Vec::new())
			.collect::<Vec<_>>();
		let mut edge_map_out = (0..self.nodes.len())
			.map(|_| Vec::new())
			.collect::<Vec<_>>();
		for (i, x) in self.edges.iter().enumerate() {
			edge_map_out[usize::from(x.0)].push(GraphEdgeIdx(i));
			edge_map_in[usize::from(x.1)].push(GraphEdgeIdx(i));
		}

		FinalizedGraph {
			graph: self,
			edge_map_in,
			edge_map_out,
		}
	}

	/// Add a node to this graph.
	#[inline]
	pub fn add_node(&mut self, node: NodeType) -> GraphNodeIdx {
		let i = self.nodes.len();
		self.nodes.push(node);
		GraphNodeIdx(i)
	}

	/// Get a node by index
	#[inline]
	pub fn get_node(&self, node_idx: GraphNodeIdx) -> &NodeType {
		self.nodes.get(usize::from(node_idx)).unwrap()
	}

	/// Get a node by index
	#[inline]
	pub fn get_node_mut(&mut self, node_idx: GraphNodeIdx) -> &mut NodeType {
		self.nodes.get_mut(usize::from(node_idx)).unwrap()
	}

	/// The number of nodes in this graph
	#[inline]
	pub fn len_nodes(&self) -> usize {
		self.nodes.len()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_nodes(&self) -> impl Iterator<Item = &NodeType> {
		self.nodes.iter()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut NodeType> {
		self.nodes.iter_mut()
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_nodes_idx(&self) -> impl Iterator<Item = (GraphNodeIdx, &NodeType)> {
		self.iter_nodes()
			.enumerate()
			.map(|(a, b)| (GraphNodeIdx(a), b))
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_nodes_idx_mut(&mut self) -> impl Iterator<Item = (GraphNodeIdx, &mut NodeType)> {
		self.iter_nodes_mut()
			.enumerate()
			.map(|(a, b)| (GraphNodeIdx(a), b))
	}

	/// Add an edge to this graph
	#[inline]
	pub fn add_edge(
		&mut self,
		from: GraphNodeIdx,
		to: GraphNodeIdx,
		edge_value: EdgeType,
	) -> GraphNodeIdx {
		let i = self.nodes.len();
		self.edges.push((from, to, edge_value));
		GraphNodeIdx(i)
	}

	/// Get an edge by index
	#[inline]
	pub fn get_edge(&self, edge_idx: GraphEdgeIdx) -> (GraphNodeIdx, GraphNodeIdx, &EdgeType) {
		self.edges
			.get(usize::from(edge_idx))
			.map(|(f, t, v)| (*f, *t, v))
			.unwrap()
	}

	/// Get an edge by index
	#[inline]
	pub fn get_edge_mut(
		&mut self,
		edge_idx: GraphEdgeIdx,
	) -> (GraphNodeIdx, GraphNodeIdx, &mut EdgeType) {
		self.edges
			.get_mut(usize::from(edge_idx))
			.map(|(f, t, v)| (*f, *t, v))
			.unwrap()
	}

	/// The number of edges in this graph
	#[inline]
	pub fn len_edges(&self) -> usize {
		self.edges.len()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_edges(&self) -> impl Iterator<Item = (GraphNodeIdx, GraphNodeIdx, &EdgeType)> {
		self.edges.iter().map(|(f, t, v)| (*f, *t, v))
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_edges_mut(
		&mut self,
	) -> impl Iterator<Item = (GraphNodeIdx, GraphNodeIdx, &mut EdgeType)> {
		self.edges.iter_mut().map(|(f, t, v)| (*f, *t, v))
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_edges_idx(
		&self,
	) -> impl Iterator<Item = (GraphEdgeIdx, (GraphNodeIdx, GraphNodeIdx, &EdgeType))> {
		self.iter_edges()
			.enumerate()
			.map(|(a, b)| (GraphEdgeIdx(a), b))
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_edges_idx_mut(
		&mut self,
	) -> impl Iterator<Item = (GraphEdgeIdx, (GraphNodeIdx, GraphNodeIdx, &mut EdgeType))> {
		self.iter_edges_mut()
			.enumerate()
			.map(|(a, b)| (GraphEdgeIdx(a), b))
	}

	/// Returns `true` if this graph has a cycle.
	#[inline]
	pub fn has_cycle(&self) -> bool {
		let mut fake_graph = GraphMap::<usize, (), Directed>::new();
		for (from, to, _) in self.iter_edges() {
			fake_graph.add_edge(from.into(), to.into(), ());
		}
		toposort(&fake_graph, None).is_err()
	}
}
