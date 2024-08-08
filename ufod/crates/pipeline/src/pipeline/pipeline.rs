//! Core pipeline structs

use std::{fmt::Debug, sync::Arc};

use crate::{
	api::{PipelineNode, PipelineNodeStub},
	graph::{finalized::FinalizedGraph, util::GraphNodeIdx},
	labels::{PipelineName, PipelineNodeID},
};

use super::syntax::{builder::PipelineBuilder, spec::PipelineSpec};

/// A node in a pipeline graph
#[derive(Debug)]
pub struct PipelineNodeData<NodeStubType: PipelineNodeStub> {
	/// The node's id
	pub id: PipelineNodeID,

	/// The node's type
	pub node_type: NodeStubType,
}

/// An edge in a pipeline graph
#[derive(Debug, Clone)]
pub enum PipelineEdgeData {
	/// A edge from an output port to an input port.
	/// PTP edges carry data between nodes.
	///
	/// Contents are (from_port, to_port)
	PortToPort((usize, usize)),

	/// An edge from a node to a node, specifying
	/// that the second *must* wait for the first.
	After,
}

impl PipelineEdgeData {
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
pub struct Pipeline<NodeStubType: PipelineNodeStub> {
	/// This pipeline's name.
	/// Must be unique.
	pub(crate) name: PipelineName,

	pub(crate) input_node_idx: GraphNodeIdx,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<PipelineNodeData<NodeStubType>, PipelineEdgeData>,
}

impl<NodeStubType: PipelineNodeStub> Pipeline<NodeStubType> {
	/// Load a pipeline from a TOML string
	pub fn from_toml_str(
		pipeline_name: &PipelineName,
		toml_str: &str,
		context: Arc<<NodeStubType::NodeType as PipelineNode>::NodeContext>,
	) -> Result<Self, ()> {
		let spec: PipelineSpec<NodeStubType> = toml::from_str(toml_str).unwrap();
		let built = PipelineBuilder::build(context, pipeline_name, spec).unwrap();
		Ok(built)
	}

	/// Iterate over all nodes in this pipeline
	pub fn iter_node_ids(&self) -> impl Iterator<Item = &PipelineNodeID> {
		self.graph.iter_nodes().map(|n| &n.id)
	}

	/// Get this pipeline's name
	pub fn get_name(&self) -> &PipelineName {
		&self.name
	}

	/// Get a node by name
	pub fn get_node(&self, node_id: &PipelineNodeID) -> Option<&NodeStubType> {
		self.graph
			.iter_nodes()
			.find(|n| n.id == *node_id)
			.map(|x| &x.node_type)
	}

	/// Get this pipeline's input node's id
	pub fn input_node_id(&self) -> &PipelineNodeID {
		&self.graph.get_node(self.input_node_idx).id
	}
}
