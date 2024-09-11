use std::fmt::Debug;

use super::{
	graph::Graph,
	util::{GraphEdgeIdx, GraphNodeIdx},
};

/// An immutable directed graph with parallel edges.
/// This is guaranteed to have no (directed) cycles.
///
/// All read operations are fast.
pub struct FinalizedGraph<NodeType, EdgeType>
where
	NodeType: Debug,
	EdgeType: Debug,
{
	/// The graph data
	pub(super) graph: Graph<NodeType, EdgeType>,

	/// An array of edge idx, sorted by start node.
	/// Redundant, but makes reads faster.
	pub(super) edge_map_out: Vec<Vec<GraphEdgeIdx>>,

	/// An array of edge idx, sorted by end node.
	/// Redundant, but makes reads faster.
	pub(super) edge_map_in: Vec<Vec<GraphEdgeIdx>>,
}

impl<NodeType, EdgeType> Debug for FinalizedGraph<NodeType, EdgeType>
where
	NodeType: Debug,
	EdgeType: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FinalizedGraph")
			.field("nodes", &self.graph.nodes)
			.field("edges", &self.graph.edges)
			.finish()
	}
}

#[allow(dead_code)]
impl<NodeType, EdgeType> FinalizedGraph<NodeType, EdgeType>
where
	NodeType: Debug,
	EdgeType: Debug,
{
	/// Get a node by index
	#[inline]
	pub fn get_node(&self, node_idx: GraphNodeIdx) -> &NodeType {
		self.graph.get_node(node_idx)
	}

	/// Get a node by index
	#[inline]
	pub fn get_node_mut(&mut self, node_idx: GraphNodeIdx) -> &mut NodeType {
		self.graph.get_node_mut(node_idx)
	}

	/// The number of nodes in this graph
	#[inline]
	pub fn len_nodes(&self) -> usize {
		self.graph.len_nodes()
	}

	/// Iterate over all nodes this graph
	#[inline]
	pub fn iter_nodes(&self) -> impl Iterator<Item = &NodeType> {
		self.graph.iter_nodes()
	}

	/// Iterate over all nodes this graph
	#[inline]
	pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut NodeType> {
		self.graph.iter_nodes_mut()
	}

	/// Iterate over all nodes in this graph, including edge index
	#[inline]
	pub fn iter_nodes_idx(&self) -> impl Iterator<Item = (GraphNodeIdx, &NodeType)> {
		self.graph.iter_nodes_idx()
	}

	/// Iterate over all nodes in this graph, including edge index
	#[inline]
	pub fn iter_nodes_idx_mut(&mut self) -> impl Iterator<Item = (GraphNodeIdx, &mut NodeType)> {
		self.graph.iter_nodes_idx_mut()
	}

	/// Get a node by index
	#[inline]
	pub fn get_edge(&self, edge_idx: GraphEdgeIdx) -> (GraphNodeIdx, GraphNodeIdx, &EdgeType) {
		self.graph.get_edge(edge_idx)
	}

	/// Get a node by index
	#[inline]
	pub fn get_edge_mut(
		&mut self,
		edge_idx: GraphEdgeIdx,
	) -> (GraphNodeIdx, GraphNodeIdx, &mut EdgeType) {
		self.graph.get_edge_mut(edge_idx)
	}

	/// The number of edges in this graph
	#[inline]
	pub fn len_edges(&self) -> usize {
		self.graph.len_edges()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_edges(&self) -> impl Iterator<Item = (GraphNodeIdx, GraphNodeIdx, &EdgeType)> {
		self.graph.iter_edges()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_edges_mut(
		&mut self,
	) -> impl Iterator<Item = (GraphNodeIdx, GraphNodeIdx, &mut EdgeType)> {
		self.graph.iter_edges_mut()
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_edges_idx(
		&self,
	) -> impl Iterator<Item = (GraphEdgeIdx, (GraphNodeIdx, GraphNodeIdx, &EdgeType))> {
		self.graph.iter_edges_idx()
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_edges_idx_mut(
		&mut self,
	) -> impl Iterator<Item = (GraphEdgeIdx, (GraphNodeIdx, GraphNodeIdx, &mut EdgeType))> {
		self.graph.iter_edges_idx_mut()
	}

	/// Get all edges starting at the given node
	pub fn edges_starting_at(&self, node: GraphNodeIdx) -> &[GraphEdgeIdx] {
		self.edge_map_out.get(usize::from(node)).unwrap()
	}

	/// Get all edges ending at the given node
	pub fn edges_ending_at(&self, node: GraphNodeIdx) -> &[GraphEdgeIdx] {
		self.edge_map_in.get(usize::from(node)).unwrap()
	}
}
