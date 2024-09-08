//! A user-provided pipeline specification

use copper_util::graph::graph::Graph;
use std::{collections::HashMap, marker::PhantomData};
use tracing::{debug, trace};

use super::{errors::PipelineBuildError, spec::PipelineSpec};
use crate::{
	base::{PipelineData, PipelineDataStub, PipelineJobContext},
	dispatcher::NodeDispatcher,
	labels::PipelineName,
	pipeline::{
		pipeline::{Pipeline, PipelineEdgeData, PipelineNodeData},
		syntax::spec::EdgeType,
	},
};

pub(in super::super) fn build_pipeline<
	DataType: PipelineData,
	ContextType: PipelineJobContext<DataType>,
>(
	context: &ContextType,
	dispatcher: &NodeDispatcher<DataType, ContextType>,
	name: &PipelineName,
	spec: &PipelineSpec<DataType>,
) -> Result<Pipeline<DataType, ContextType>, PipelineBuildError<DataType>> {
	debug!(message = "Building pipeline", pipeline_name = ?name);

	// Initialize all variables
	let mut graph = Graph::new();
	let mut node_output_name_map_ptp = HashMap::new();
	let mut node_input_name_map_ptp = HashMap::new();
	let mut node_output_name_map_after = HashMap::new();
	let mut node_input_name_map_after = HashMap::new();

	// Add nodes to the graph
	trace!(message = "Making nodes", pipeline_name = ?name);
	for (node_id, node_spec) in &spec.nodes {
		let n = graph.add_node(PipelineNodeData {
			id: node_id.clone(),
			node_params: node_spec.data.params.clone(),
			node_type: node_spec.data.node_type.clone(),
		});

		node_output_name_map_ptp.insert(node_id.clone(), n);
		node_input_name_map_ptp.insert(node_id.clone(), n);
		node_output_name_map_after.insert(node_id.clone(), n);
		node_input_name_map_after.insert(node_id.clone(), n);
	}

	// Make sure all "after" edges are valid and create them in the graph.
	trace!(message = "Making `after` edges", pipeline_name = ?name);
	for (edge_id, edge_spec) in spec
		.edges
		.iter()
		.filter(|(_, v)| matches!(v.data.edge_type, EdgeType::After))
	{
		let source = node_input_name_map_after
			.get(&edge_spec.source.node)
			.ok_or(PipelineBuildError::NoNode {
				edge_id: edge_id.clone(),
				invalid_node_id: edge_spec.source.node.clone(),
			})?;
		let target = node_input_name_map_after
			.get(&edge_spec.target.node)
			.ok_or(PipelineBuildError::NoNode {
				edge_id: edge_id.clone(),
				invalid_node_id: edge_spec.target.node.clone(),
			})?;

		graph.add_edge(source.clone(), target.clone(), PipelineEdgeData::After);
	}

	// Make sure all "data" edges are valid and create them in the graph.
	trace!(message = "Making `data` edges", pipeline_name = ?name);
	for (edge_id, edge_spec) in spec
		.edges
		.iter()
		.filter(|(_, v)| matches!(v.data.edge_type, EdgeType::Data))
	{
		let source_node =
			spec.nodes
				.get(&edge_spec.source.node)
				.ok_or(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.source.node.clone(),
				})?;
		let target_node =
			spec.nodes
				.get(&edge_spec.target.node)
				.ok_or(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.target.node.clone(),
				})?;

		let source_node_info = dispatcher
			.node_info(
				context,
				&source_node.data.node_type,
				&source_node.data.params,
				edge_spec.source.node.id(),
			)?
			.unwrap();

		let target_node_info = dispatcher
			.node_info(
				context,
				&target_node.data.node_type,
				&target_node.data.params,
				edge_spec.target.node.id(),
			)?
			.unwrap();

		// Make sure types are compatible
		{
			let source_type = *source_node_info
				.outputs()
				.get(&edge_spec.source.port)
				.ok_or(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.source.node.clone(),
				})?;

			let target_type = *target_node_info
				.outputs()
				.get(&edge_spec.target.port)
				.ok_or(PipelineBuildError::NoNode {
					edge_id: edge_id.clone(),
					invalid_node_id: edge_spec.target.node.clone(),
				})?;

			if !source_type.is_subset_of(&target_type) {
				return Err(PipelineBuildError::TypeMismatch {
					edge_id: edge_id.clone(),
					source_type,
					target_type,
				});
			}
		}

		if !source_node_info
			.inputs()
			.contains_key(&edge_spec.source.port)
		{
			return Err(PipelineBuildError::NoSuchOutputPort {
				edge_id: edge_id.clone(),
				node: edge_spec.source.node.clone(),
				invalid_port: edge_spec.source.port.clone(),
			});
		};

		if !target_node_info
			.inputs()
			.contains_key(&edge_spec.target.port)
		{
			return Err(PipelineBuildError::NoSuchOutputPort {
				edge_id: edge_id.clone(),
				node: edge_spec.target.node.clone(),
				invalid_port: edge_spec.target.port.clone(),
			});
		};

		let source_node_idx = *node_output_name_map_ptp
			.get(&edge_spec.source.node)
			.unwrap();

		let target_node_idx = *node_input_name_map_ptp.get(&edge_spec.target.node).unwrap();

		// Create the edge
		graph.add_edge(
			source_node_idx,
			target_node_idx,
			PipelineEdgeData::PortToPort((
				edge_spec.source.port.clone(),
				edge_spec.target.port.clone(),
			)),
		);
	}

	trace!(message = "Looking for cycles", pipeline_name = ?name);
	// Make sure our graph doesn't have any cycles
	if graph.has_cycle() {
		return Err(PipelineBuildError::HasCycle);
	}

	return Ok(Pipeline {
		_pa: PhantomData {},
		_pb: PhantomData {},
		name: name.clone(),
		graph: graph.finalize(),
	});
}
