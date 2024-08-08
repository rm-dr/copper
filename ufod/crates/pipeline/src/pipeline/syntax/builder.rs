//! A user-provided pipeline specification

use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tracing::{debug, trace};

use super::{
	errors::{PipelineErrorNode, PipelinePrepareError},
	ports::{NodeInput, NodeOutput},
	spec::PipelineSpec,
};
use crate::{
	api::{PipelineNode, PipelineNodeStub},
	graph::{graph::Graph, util::GraphNodeIdx},
	labels::{PipelineName, PipelineNodeID, PipelinePortID},
	pipeline::pipeline::{Pipeline, PipelineEdgeData, PipelineNodeData},
	SDataStub,
};

pub(in super::super) struct PipelineBuilder<NodeStubType: PipelineNodeStub> {
	/// The name of the pipeline we're building
	name: PipelineName,

	/// The context with which to build this pipeline
	context: Arc<<NodeStubType::NodeType as PipelineNode>::NodeContext>,

	/// The pipeline spec to build
	spec: PipelineSpec<NodeStubType>,

	/// The pipeline graph we're building
	graph: RefCell<Graph<PipelineNodeData<NodeStubType>, PipelineEdgeData>>,

	/// The index of this pipeline's input node
	input_node_idx: GraphNodeIdx,

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

impl<'a, NodeStubType: PipelineNodeStub> PipelineBuilder<NodeStubType> {
	pub fn build(
		context: Arc<<NodeStubType::NodeType as PipelineNode>::NodeContext>,
		name: &PipelineName,
		spec: PipelineSpec<NodeStubType>,
	) -> Result<Pipeline<NodeStubType>, PipelinePrepareError<NodeStubType>> {
		debug!(message = "Building pipeline", pipeline_name = ?name);

		// Initialize all variables
		let builder = {
			let mut graph = Graph::new();

			// Add input and output nodes to the graph
			let input_node_idx = graph.add_node(PipelineNodeData {
				id: PipelineNodeID::new("INPUT"),
				node_type: spec.input.clone(),
			});

			Self {
				name: name.clone(),
				context,
				spec,
				graph: RefCell::new(graph),
				input_node_idx,
				node_output_name_map_ptp: RefCell::new(HashMap::new()),
				node_input_name_map_ptp: RefCell::new(HashMap::new()),
				node_output_name_map_after: RefCell::new(HashMap::new()),
				node_input_name_map_after: RefCell::new(HashMap::new()),
			}
		};

		// Make sure every node's inputs are valid,
		// create the corresponding edges in the graph.
		trace!(message = "Checking inputs", pipeline_name = ?name);
		{
			for (node_id, node_spec) in &builder.spec.nodes {
				for (input_name, out_link) in &node_spec.inputs {
					builder.check_link(
						out_link,
						&NodeInput::Node {
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
			// If this is a normal node, just add it.
			let n = builder.graph.borrow_mut().add_node(PipelineNodeData {
				id: node_name.clone(),
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
					let in_port = builder.get_input(&node_spec.node_type, input_name, node_name)?;
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
			name: builder.name,
			graph: builder.graph.into_inner().finalize(),
			input_node_idx: builder.input_node_idx,
		});
	}

	/// Is the port `input_port_id` of a node with type `node_type`
	/// compatible with the given input?
	#[inline(always)]
	fn is_input_compatible(
		&self,
		node_type: &NodeStubType,
		input_port_id: &PipelinePortID,
		input_type: SDataStub<NodeStubType>,
		// Only used for errors
		node_id: &PipelineNodeID,
	) -> Result<bool, PipelinePrepareError<NodeStubType>> {
		Ok({
			let idx = node_type
				.input_with_name(&self.context, input_port_id)
				.map_err(|error| PipelinePrepareError::NodeStubError { error })?
				.ok_or(PipelinePrepareError::NoNodeInput {
					node: PipelineErrorNode::Named(node_id.clone()),
					input: input_port_id.clone(),
				})?;
			node_type
				.input_compatible_with(&self.context, idx, input_type)
				.map_err(|error| PipelinePrepareError::NodeStubError { error })?
		})
	}

	/// Find the port index of the input port `input_port_id`
	/// of a node with type `node_type`.
	#[inline(always)]
	fn get_input(
		&self,
		node_type: &NodeStubType,
		input_port_id: &PipelinePortID,

		// Only used for errors
		node_id: &PipelineNodeID,
	) -> Result<usize, PipelinePrepareError<NodeStubType>> {
		match node_type {
			t => t.input_with_name(&self.context, input_port_id),
		}
		.map_err(|error| PipelinePrepareError::NodeStubError { error })?
		.ok_or(PipelinePrepareError::NoNodeInput {
			node: PipelineErrorNode::Named(node_id.clone()),
			input: input_port_id.clone(),
		})
	}

	/// Find the port index and type of the output port `output_port_id`
	/// of a node with type `node_type`.
	///
	/// This provides both `get_input` and `is_input_compatible` for outputs.
	/// Outputs are simpler than inputs---they always have exactly one type.
	#[inline(always)]
	fn get_output(
		&self,
		node_type: &NodeStubType,
		output_port_id: &PipelinePortID,

		// Only used for errors
		node_id: &PipelineNodeID,
	) -> Result<(usize, SDataStub<NodeStubType>), PipelinePrepareError<NodeStubType>> {
		match node_type {
			t => {
				let idx = t
					.output_with_name(&self.context, output_port_id)
					.map_err(|error| PipelinePrepareError::NodeStubError { error })?
					.ok_or(PipelinePrepareError::NoNodeOutput {
						output: output_port_id.clone(),
						node: PipelineErrorNode::Named(node_id.clone()),
					})?;
				let output_type = t
					.output_type(&self.context, idx)
					.map_err(|error| PipelinePrepareError::NodeStubError { error })?;
				Ok((idx, output_type))
			}
		}
	}

	/// Connect `out_link` to port index `in_port` of node `node_idx`.
	#[allow(clippy::too_many_arguments)]
	fn add_to_graph(
		&self,
		in_port: usize,
		node_idx: GraphNodeIdx,
		out_link: &NodeOutput<NodeStubType>,
	) -> Result<(), PipelinePrepareError<NodeStubType>> {
		match out_link {
			NodeOutput::Pipeline { port } => {
				let out_port = self
					.spec
					.input
					.output_with_name(&self.context, port)
					.map_err(|error| PipelinePrepareError::NodeStubError { error })?
					.unwrap();
				self.graph.borrow_mut().add_edge(
					self.input_node_idx,
					node_idx,
					PipelineEdgeData::PortToPort((out_port, in_port)),
				);
			}
			NodeOutput::Inline(node) => {
				let x = self.graph.borrow_mut().add_node(PipelineNodeData {
					id: PipelineNodeID::new("INLINE"),
					node_type: node.clone(),
				});
				self.graph.borrow_mut().add_edge(
					x,
					node_idx,
					PipelineEdgeData::PortToPort((0, in_port)),
				);
			}
			NodeOutput::Node { node, port } => {
				let out_port = self
					.get_output(&self.spec.nodes.get(node).unwrap().node_type, port, node)?
					.0;
				self.graph.borrow_mut().add_edge(
					*self.node_output_name_map_ptp.borrow().get(node).unwrap(),
					node_idx,
					PipelineEdgeData::PortToPort((out_port, in_port)),
				);
			}
		}

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
		output: &NodeOutput<NodeStubType>,
		input: &NodeInput,
	) -> Result<(), PipelinePrepareError<NodeStubType>> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type: SDataStub<NodeStubType> = match output {
			NodeOutput::Inline(node) => {
				// Inline nodes must have exactly one output
				if node
					.n_outputs(&self.context)
					.map_err(|error| PipelinePrepareError::NodeStubError { error })?
					!= 1
				{
					return Err(PipelinePrepareError::BadInlineNode {
						input: input.clone(),
					});
				}
				node.output_type(&self.context, 0)
					.map_err(|error| PipelinePrepareError::NodeStubError { error })?
			}

			NodeOutput::Pipeline { port } => {
				if let Some(idx) = self
					.spec
					.input
					.output_with_name(&self.context, port)
					.map_err(|error| PipelinePrepareError::NodeStubError { error })?
				{
					self.spec
						.input
						.output_type(&self.context, idx)
						.map_err(|error| PipelinePrepareError::NodeStubError { error })?
				} else {
					return Err(PipelinePrepareError::NoNodeOutput {
						node: PipelineErrorNode::PipelineInput,
						output: port.clone(),
					});
				}
			}

			NodeOutput::Node { node, port } => {
				let get_node = self.spec.nodes.get(node);

				if get_node.is_none() {
					return Err(PipelinePrepareError::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					});
				}
				let get_node = get_node.unwrap();
				self.get_output(&get_node.node_type, port, node)?.1
			}
		};

		// Find the datatype of the input port we're connecting to.
		// While doing this, make sure both the input node and port exist.
		let compatible = match &input {
			NodeInput::Node { node, port } => {
				let get_node = self.spec.nodes.get(node);

				if get_node.is_none() {
					return Err(PipelinePrepareError::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					});
				}
				let get_node = get_node.unwrap();
				self.is_input_compatible(&get_node.node_type, port, output_type, node)?
			}
		};

		if !compatible {
			return Err(PipelinePrepareError::TypeMismatch {
				output: match output {
					NodeOutput::Node { node, port } => {
						(PipelineErrorNode::Named(node.clone()), port.clone())
					}
					NodeOutput::Inline(_) => {
						(PipelineErrorNode::Inline, PipelinePortID::new("INLINE"))
					}
					NodeOutput::Pipeline { port } => {
						(PipelineErrorNode::PipelineInput, port.clone())
					}
				},
				output_type,
				input: input.clone(),
			});
		}

		return Ok(());
	}
}
