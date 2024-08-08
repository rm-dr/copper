//! Core pipeline structs

use std::{fmt::Debug, sync::Arc};

use crate::{
	api::{PipelineNode, PipelineNodeStub},
	graph::{finalized::FinalizedGraph, util::GraphNodeIdx},
	labels::{PipelineLabel, PipelineNodeLabel},
};

use super::syntax::{builder::PipelineBuilder, internalnode::InternalNodeStub, spec::PipelineSpec};

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

/// A fully loaded data processing pipeline.
#[derive(Debug)]
pub struct Pipeline<StubType: PipelineNodeStub> {
	/// This pipeline's name.
	/// Must be unique.
	pub(crate) name: PipelineLabel,

	pub(crate) input_node_idx: GraphNodeIdx,
	pub(crate) output_node_idx: GraphNodeIdx,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<(PipelineNodeLabel, InternalNodeStub<StubType>), PipelineEdge>,
}

impl<StubType: PipelineNodeStub> Pipeline<StubType> {
	/// Load a pipeline from a TOML string
	pub fn from_toml_str(
		pipeline_name: &str,
		toml_str: &str,
		context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
	) -> Result<Self, ()> {
		let spec: PipelineSpec<StubType> = toml::from_str(toml_str).unwrap();
		let built = PipelineBuilder::build(context, &vec![], pipeline_name, spec).unwrap();
		Ok(built)
	}

	/// Iterate over all nodes in this pipeline
	pub fn iter_node_labels(&self) -> impl Iterator<Item = &PipelineNodeLabel> {
		self.graph.iter_nodes().map(|(l, _)| l)
	}

	/// Get this pipeline's name
	pub fn get_name(&self) -> &PipelineLabel {
		&self.name
	}

	/// Get a node by name
	pub fn get_node(&self, node_label: &PipelineNodeLabel) -> &StubType {
		let x = &self
			.graph
			.iter_nodes()
			.find(|(l, _)| l == node_label)
			.unwrap()
			.1;

		match x {
			InternalNodeStub::Pipeline { .. } => unreachable!(),
			InternalNodeStub::User(x) => &x,
		}
	}

	/// Get this pipeline's input node's label
	pub fn input_node_label(&self) -> &PipelineNodeLabel {
		&self.graph.get_node(self.input_node_idx).0
	}

	/// Get this pipeline's output node's label
	pub fn output_node_label(&self) -> &PipelineNodeLabel {
		&self.graph.get_node(self.input_node_idx).0
	}
}
