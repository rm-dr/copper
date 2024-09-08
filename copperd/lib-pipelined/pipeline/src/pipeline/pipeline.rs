//! Core pipeline structs

use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug, marker::PhantomData};

use super::syntax::{build::build_pipeline, errors::PipelineBuildError, spec::PipelineSpec};
use crate::{
	api::{PipelineData, PipelineJobContext},
	dispatcher::{NodeDispatcher, NodeParameterValue},
	graph::finalized::FinalizedGraph,
	labels::{PipelineName, PipelineNodeID, PipelinePortID},
	nodes::input::INPUT_NODE_TYPE_NAME,
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
	PortToPort((PipelinePortID, PipelinePortID)),

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

/// A fully loaded data processing pipeline.
#[derive(Debug)]
pub struct Pipeline<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	pub(crate) _pa: PhantomData<DataType>,
	pub(crate) _pb: PhantomData<ContextType>,

	/// This pipeline's name.
	/// Must be unique.
	pub(crate) name: PipelineName,

	/// This pipeline's node graph
	pub(crate) graph: FinalizedGraph<PipelineNodeData<DataType>, PipelineEdgeData>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	Pipeline<DataType, ContextType>
{
	/// Load a pipeline from a TOML string
	pub fn build(
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		context: &ContextType,
		pipeline_name: &PipelineName,
		pipeline_spec: &PipelineSpec<DataType>,
	) -> Result<Self, PipelineBuildError<DataType>> {
		let built = build_pipeline(context, dispatcher, pipeline_name, pipeline_spec)?;
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

	/// Get this pipeline's input node ids
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
