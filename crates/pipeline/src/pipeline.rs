use smartstring::{LazyCompact, SmartString};
use std::fmt::Debug;
use ufo_util::graph::FinalizedGraph;

use crate::{
	nodes::nodetype::PipelineNodeType,
	syntax::{labels::PipelineNodeLabel, spec::PipelineConfig},
};

/// An edge in a pipeline
#[derive(Debug, Clone)]
pub(super) enum PipelineEdge {
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
pub struct Pipeline {
	/// This pipeline's name.
	/// Must be unique.
	pub(crate) name: SmartString<LazyCompact>,

	/// This pipeline's configuration
	pub(crate) config: PipelineConfig,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<(PipelineNodeLabel, PipelineNodeType), PipelineEdge>,
}

impl Pipeline {
	/// Get this pipeline's configuration
	pub fn get_config(&self) -> &PipelineConfig {
		&self.config
	}
}
