use std::fmt::Debug;

use crate::{
	api::PipelineNodeStub,
	graph::{finalized::FinalizedGraph, util::GraphNodeIdx},
	labels::{PipelineLabel, PipelineNodeLabel},
	syntax::internalnode::InternalNodeStub,
};

/// An edge in a pipeline
#[derive(Debug, Clone)]
pub enum PipelineEdge {
	/// A edge from an output port to an input port.
	/// PTP edges carry data between nodes.
	///
	/// Contents are (from_port, to_port)
	PortToPort((usize, usize)),

	/// An edge from a node to a node, specifying
	/// that the second *must* wait for the first.
	After,
}

impl PipelineEdge {
	/// Is this a `Self::PortToPort`?
	pub fn is_ptp(&self) -> bool {
		matches!(self, Self::PortToPort(_))
	}

	/// Is this a `Self::After`?
	pub fn is_after(&self) -> bool {
		matches!(self, Self::After)
	}

	/// Get the port this edge starts at
	pub fn source_port(&self) -> Option<usize> {
		match self {
			Self::PortToPort((s, _)) => Some(*s),
			Self::After => None,
		}
	}

	/// Get the port this edge ends at
	pub fn target_port(&self) -> Option<usize> {
		match self {
			Self::PortToPort((_, t)) => Some(*t),
			Self::After => None,
		}
	}
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc.
#[derive(Debug)]
pub(crate) struct Pipeline<StubType: PipelineNodeStub> {
	/// This pipeline's name.
	/// Must be unique.
	pub(crate) name: PipelineLabel,

	pub(crate) input_node_idx: GraphNodeIdx,
	pub(crate) output_node_idx: GraphNodeIdx,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<(PipelineNodeLabel, InternalNodeStub<StubType>), PipelineEdge>,
}
