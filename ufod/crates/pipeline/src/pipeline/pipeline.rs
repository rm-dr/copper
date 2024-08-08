//! Core pipeline structs

use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug, marker::PhantomData};

use super::syntax::{builder::PipelineBuilder, errors::PipelinePrepareError, spec::PipelineSpec};
use crate::{
	api::{PipelineData, PipelineJobContext},
	dispatcher::{NodeDispatcher, NodeParameterValue},
	graph::{finalized::FinalizedGraph, util::GraphNodeIdx},
	labels::{PipelineName, PipelineNodeID},
};

/// A node in a pipeline graph
#[derive(Debug)]
pub struct PipelineNodeData<DataType: PipelineData> {
	/// The node's id
	pub id: PipelineNodeID,

	/// This node's type
	pub node_type: SmartString<LazyCompact>,

	/// This node's parameters
	pub node_params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue<DataType>>,
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
pub struct Pipeline<DataType: PipelineData, ContextType: PipelineJobContext> {
	pub(crate) _pa: PhantomData<DataType>,
	pub(crate) _pb: PhantomData<ContextType>,

	/// This pipeline's name.
	/// Must be unique.
	pub(crate) name: PipelineName,

	pub(crate) input_node_idx: GraphNodeIdx,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<PipelineNodeData<DataType>, PipelineEdgeData>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext> Pipeline<DataType, ContextType> {
	/// Load a pipeline from a TOML string
	pub fn from_toml_str(
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		context: &ContextType,
		pipeline_name: &PipelineName,
		toml_str: &str,
	) -> Result<Self, PipelinePrepareError<DataType>> {
		let spec: PipelineSpec<DataType> = toml::from_str(toml_str).unwrap();
		let built = PipelineBuilder::build(context, dispatcher, pipeline_name, spec)?;
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
	pub fn get_node(&self, node_id: &PipelineNodeID) -> Option<&PipelineNodeData<DataType>> {
		self.graph.iter_nodes().find(|n| n.id == *node_id)
	}

	/// Get this pipeline's input node's id
	pub fn input_node_id(&self) -> &PipelineNodeID {
		&self.graph.get_node(self.input_node_idx).id
	}
}
