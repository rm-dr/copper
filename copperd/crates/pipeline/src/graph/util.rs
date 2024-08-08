/// The index of a node in a [`Graph`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphNodeIdx(pub(super) usize);

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

	/// Make this index from a `usize`
	pub fn from_usize(value: usize) -> Self {
		GraphNodeIdx(value)
	}
}

/// The index of an edge in a [`Graph`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphEdgeIdx(pub(super) usize);

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
