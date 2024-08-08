//! A user-provided pipeline specification

use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, marker::PhantomData};
use tracing::{debug, trace};

use super::{
	errors::PipelinePrepareError,
	ports::{NodeInput, NodeOutput},
	spec::PipelineSpec,
};
use crate::{
	api::{PipelineData, PipelineDataStub, PipelineJobContext},
	dispatcher::NodeDispatcher,
	graph::{graph::Graph, util::GraphNodeIdx},
	labels::{PipelineName, PipelineNodeID, PipelinePortID},
	pipeline::pipeline::{Pipeline, PipelineEdgeData, PipelineNodeData},
};

pub(in super::super) struct PipelineBuilder<
	'a,
	DataType: PipelineData,
	ContextType: PipelineJobContext<DataType>,
> {
	_pa: PhantomData<DataType>,
	_pb: PhantomData<ContextType>,
	context: &'a ContextType,
	dispatcher: &'a NodeDispatcher<DataType, ContextType>,

	/// The name of the pipeline we're building
	name: PipelineName,

	/// The pipeline spec to build
	spec: PipelineSpec<DataType>,

	/// The pipeline graph we're building
	graph: RefCell<Graph<PipelineNodeData<DataType>, PipelineEdgeData>>,

	/// Map node names to node indices
	/// (used when connecting port-to-port outputs)
	node_output_name_map_ptp: RefCell<HashMap<PipelineNodeID, GraphNodeIdx>>,

	/// Map node names to node indices
	/// (used when connecting port-to-port inputs)
	node_input_name_map_ptp: RefCell<HashMap<PipelineNodeID, GraphNodeIdx>>,

	/// Map node names to node indices
	/// (used when connecting "after" outputs)
	node_output_name_map_after: RefCell<HashMap<PipelineNodeID, GraphNodeIdx>>,

	/// Map node names to node indices
	/// (used when connecting "after" inputs)
	node_input_name_map_after: RefCell<HashMap<PipelineNodeID, GraphNodeIdx>>,
}

impl<'a, DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	PipelineBuilder<'a, DataType, ContextType>
{
	pub fn build(
		context: &'a ContextType,
		dispatcher: &'a NodeDispatcher<DataType, ContextType>,
		name: &PipelineName,
		spec: PipelineSpec<DataType>,
	) -> Result<Pipeline<DataType, ContextType>, PipelinePrepareError<DataType>> {
		debug!(message = "Building pipeline", pipeline_name = ?name);

		// Initialize all variables
		let builder = Self {
			_pa: PhantomData {},
			_pb: PhantomData {},
			context,
			dispatcher,

			name: name.clone(),
			spec,

			graph: RefCell::new(Graph::new()),
			node_output_name_map_ptp: RefCell::new(HashMap::new()),
			node_input_name_map_ptp: RefCell::new(HashMap::new()),
			node_output_name_map_after: RefCell::new(HashMap::new()),
			node_input_name_map_after: RefCell::new(HashMap::new()),
		};

		// Make sure every node's inputs are valid,
		// create the corresponding edges in the graph.
		trace!(message = "Checking inputs", pipeline_name = ?name);
		{
			for (node_id, node_spec) in &builder.spec.nodes {
				for (input_name, out_link) in &node_spec.inputs {
					builder.check_link(
						out_link,
						&NodeInput {
							node: node_id.clone(),
							port: input_name.clone(),
						},
					)?;
				}
			}
		}

		trace!(message = "Making nodes", pipeline_name = ?name);
		// Add nodes to the graph
		for (node_name, node_spec) in builder.spec.nodes.iter() {
			let n = builder.graph.borrow_mut().add_node(PipelineNodeData {
				id: node_name.clone(),
				node_params: node_spec.node_params.clone(),
				node_type: node_spec.node_type.clone(),
			});

			builder
				.node_output_name_map_ptp
				.borrow_mut()
				.insert(node_name.clone(), n);
			builder
				.node_input_name_map_ptp
				.borrow_mut()
				.insert(node_name.clone(), n);
			builder
				.node_output_name_map_after
				.borrow_mut()
				.insert(node_name.clone(), n);
			builder
				.node_input_name_map_after
				.borrow_mut()
				.insert(node_name.clone(), n);
		}

		trace!(message = "Making `after` edges", pipeline_name = ?name);
		// Make sure all "after" edges are valid and create them in the graph.
		for (node_name, node_spec) in &builder.spec.nodes {
			for after_name in node_spec.after.iter().unique() {
				if let Some(after_idx) = builder.node_input_name_map_after.borrow().get(after_name)
				{
					builder.graph.borrow_mut().add_edge(
						*after_idx,
						*builder
							.node_input_name_map_after
							.borrow()
							.get(node_name)
							.unwrap(),
						PipelineEdgeData::After,
					);
				} else {
					return Err(PipelinePrepareError::NoNodeAfter {
						node: after_name.clone(),
						caused_by_after_in: node_name.clone(),
					});
				}
			}
		}

		trace!(message = "Making `ptp` edges", pipeline_name = ?name);
		// Make sure all "port to port" edges are valid and create them in the graph.
		{
			for (node_name, node_spec) in &builder.spec.nodes {
				let node_idx = *builder
					.node_input_name_map_ptp
					.borrow()
					.get(node_name)
					.unwrap();
				for (input_name, out_link) in node_spec.inputs.iter() {
					let node_info = dispatcher
						.node_info(
							context,
							&node_spec.node_type,
							&node_spec.node_params,
							node_name.id(),
						)?
						.ok_or(PipelinePrepareError::InvalidNodeType {
							node: node_name.clone(),
							bad_type: node_spec.node_type.clone(),
						})?;
					let in_port = (*node_info)
						.inputs()
						.get_key_value(input_name)
						.ok_or(PipelinePrepareError::NoNodeInput {
							node: node_name.clone(),
							input: input_name.clone(),
						})?
						.0
						.clone();

					builder.add_to_graph(in_port, node_idx, out_link)?;
				}
			}
		}

		trace!(message = "Looking for cycles", pipeline_name = ?name);
		// Make sure our graph doesn't have any cycles
		if builder.graph.borrow().has_cycle() {
			return Err(PipelinePrepareError::HasCycle);
		}

		return Ok(Pipeline {
			_pa: PhantomData {},
			_pb: PhantomData {},
			name: builder.name,
			graph: builder.graph.into_inner().finalize(),
		});
	}

	/// Connect `out_link` to port index `in_port` of node `node_idx`.
	#[allow(clippy::too_many_arguments)]
	fn add_to_graph(
		&self,
		in_port: PipelinePortID,
		node_idx: GraphNodeIdx,
		out_link: &NodeOutput,
	) -> Result<(), PipelinePrepareError<DataType>> {
		let node_spec = self.spec.nodes.get(&out_link.node).unwrap();
		let node_info = self
			.dispatcher
			.node_info(
				self.context,
				&node_spec.node_type,
				&node_spec.node_params,
				out_link.node.id(),
			)?
			.ok_or(PipelinePrepareError::InvalidNodeType {
				node: out_link.node.clone(),
				bad_type: node_spec.node_type.clone(),
			})?;

		let out_port = node_info
			.outputs()
			.get_key_value(&out_link.port)
			.unwrap()
			.0
			.clone();

		self.graph.borrow_mut().add_edge(
			*self
				.node_output_name_map_ptp
				.borrow()
				.get(&out_link.node)
				.unwrap(),
			node_idx,
			PipelineEdgeData::PortToPort((out_port, in_port)),
		);

		Ok(())
	}

	/// Check a link from `output` to `input`.
	/// Returns [`PipelineCheckResult::Ok`] if everything is ok, and an error otherwise.
	///
	/// This makes sure that...
	/// - The output node exists
	/// - The output node has the specified port
	/// - The input node exists
	/// - The input node has the specified port
	/// - The input and output ports have matching types
	fn check_link(
		&self,
		output: &NodeOutput,
		input: &NodeInput,
	) -> Result<(), PipelinePrepareError<DataType>> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type: <DataType as PipelineData>::DataStubType = {
			let get_node = self.spec.nodes.get(&output.node);
			if get_node.is_none() {
				return Err(PipelinePrepareError::NoNode {
					node: output.node.clone(),
					caused_by: input.clone(),
				});
			}
			let get_node = get_node.unwrap();

			let node_info = self
				.dispatcher
				.node_info(
					self.context,
					&get_node.node_type,
					&get_node.node_params,
					output.node.id(),
				)?
				.ok_or(PipelinePrepareError::InvalidNodeType {
					node: output.node.clone(),
					bad_type: get_node.node_type.clone(),
				})?;

			*node_info
				.outputs()
				.get(&output.port)
				.ok_or(PipelinePrepareError::NoNodeOutput {
					node: output.node.clone(),
					output: output.port.clone(),
				})?
		};

		let input_type: <DataType as PipelineData>::DataStubType = {
			let get_node = self.spec.nodes.get(&input.node);
			if get_node.is_none() {
				return Err(PipelinePrepareError::NoNode {
					node: input.node.clone(),
					caused_by: input.clone(),
				});
			}
			let get_node = get_node.unwrap();

			let node_info = self
				.dispatcher
				.node_info(
					self.context,
					&get_node.node_type,
					&get_node.node_params,
					input.node.id(),
				)?
				.ok_or(PipelinePrepareError::InvalidNodeType {
					node: input.node.clone(),
					bad_type: get_node.node_type.clone(),
				})?;

			*node_info
				.inputs()
				.get(&input.port)
				.ok_or(PipelinePrepareError::NoNodeInput {
					node: input.node.clone(),
					input: input.port.clone(),
				})?
		};

		if !output_type.is_subset_of(&input_type) {
			return Err(PipelinePrepareError::TypeMismatch {
				output: (output.node.clone(), output.port.clone()),
				output_type,
				input: input.clone(),
			});
		}

		return Ok(());
	}
}
