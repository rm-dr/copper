//! Convenient graph manipulation.
//! We don't use petgraph because we need parallel edges.

use std::fmt::Debug;

/// The index of a node in a [`Graph`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphNodeIdx(usize);

impl From<GraphNodeIdx> for usize {
	fn from(value: GraphNodeIdx) -> Self {
		value.0
	}
}

impl GraphNodeIdx {
	/// Get this index as a `usize`
	pub fn as_usize(&self) -> usize {
		self.0
	}
}

/// The index of an edge in a [`Graph`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphEdgeIdx(usize);

impl From<GraphEdgeIdx> for usize {
	fn from(value: GraphEdgeIdx) -> Self {
		value.0
	}
}

impl GraphEdgeIdx {
	/// Get this index as a `usize`
	pub fn as_usize(&self) -> usize {
		self.0
	}
}

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
	nodes: Vec<NodeType>,

	/// Array of edges in this graph
	edges: Vec<(GraphNodeIdx, GraphNodeIdx, EdgeType)>,
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

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_nodes_idx(&self) -> impl Iterator<Item = (GraphNodeIdx, &NodeType)> {
		self.iter_nodes()
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
	pub fn get_edge(&self, edge_idx: GraphEdgeIdx) -> &(GraphNodeIdx, GraphNodeIdx, EdgeType) {
		self.edges.get(usize::from(edge_idx)).unwrap()
	}

	/// The number of edges in this graph
	#[inline]
	pub fn len_edges(&self) -> usize {
		self.edges.len()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_edges(&self) -> impl Iterator<Item = &(GraphNodeIdx, GraphNodeIdx, EdgeType)> {
		self.edges.iter()
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_edges_idx(
		&self,
	) -> impl Iterator<Item = (GraphEdgeIdx, &(GraphNodeIdx, GraphNodeIdx, EdgeType))> {
		self.iter_edges()
			.enumerate()
			.map(|(a, b)| (GraphEdgeIdx(a), b))
	}
}

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
	graph: Graph<NodeType, EdgeType>,

	/// An array of edge idx, sorted by start node.
	/// Redundant, but makes reads faster.
	edge_map_out: Vec<Vec<GraphEdgeIdx>>,

	/// An array of edge idx, sorted by end node.
	/// Redundant, but makes reads faster.
	edge_map_in: Vec<Vec<GraphEdgeIdx>>,
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

	/// Iterate over all nodes in this graph, including edge index
	#[inline]
	pub fn iter_nodes_idx(&self) -> impl Iterator<Item = (GraphNodeIdx, &NodeType)> {
		self.graph.iter_nodes_idx()
	}

	/// Get a node by index
	#[inline]
	pub fn get_edge(&self, edge_idx: GraphEdgeIdx) -> &(GraphNodeIdx, GraphNodeIdx, EdgeType) {
		self.graph.get_edge(edge_idx)
	}

	/// The number of edges in this graph
	#[inline]
	pub fn len_edges(&self) -> usize {
		self.graph.len_edges()
	}

	/// Iterate over all edges in this graph
	#[inline]
	pub fn iter_edges(&self) -> impl Iterator<Item = &(GraphNodeIdx, GraphNodeIdx, EdgeType)> {
		self.graph.iter_edges()
	}

	/// Iterate over all edges in this graph, including edge index
	#[inline]
	pub fn iter_edges_idx(
		&self,
	) -> impl Iterator<Item = (GraphEdgeIdx, &(GraphNodeIdx, GraphNodeIdx, EdgeType))> {
		self.graph.iter_edges_idx()
	}
}

impl<NodeType, EdgeType> FinalizedGraph<NodeType, EdgeType>
where
	NodeType: Debug,
	EdgeType: Debug,
{
	/// Get all edges starting at the given node
	pub fn edges_starting_at(&self, node: GraphNodeIdx) -> &[GraphEdgeIdx] {
		self.edge_map_out.get(usize::from(node)).unwrap()
	}

	/// Get all edges ending at the given node
	pub fn edges_ending_at(&self, node: GraphNodeIdx) -> &[GraphEdgeIdx] {
		self.edge_map_in.get(usize::from(node)).unwrap()
	}
}
