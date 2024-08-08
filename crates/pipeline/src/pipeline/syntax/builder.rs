//! A user-provided pipeline specification

use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tracing::debug;

use super::{
	errors::{PipelineErrorNode, PipelinePrepareError},
	ports::{NodeInput, NodeOutput},
	spec::PipelineSpec,
};
use crate::{
	api::{PipelineNode, PipelineNodeStub},
	graph::{graph::Graph, util::GraphNodeIdx},
	labels::{PipelineLabel, PipelineNodeLabel, PipelinePortLabel},
	pipeline::pipeline::{Pipeline, PipelineEdge},
	SDataStub,
};

pub(in super::super) struct PipelineBuilder<StubType: PipelineNodeStub> {
	/// The name of the pipeline we're building
	name: PipelineLabel,

	/// The context with which to build this pipeline
	context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,

	/// The pipeline spec to build
	spec: PipelineSpec<StubType>,

	/// The pipeline graph we're building
	graph: RefCell<Graph<(PipelineNodeLabel, StubType), PipelineEdge>>,

	/// The index of this pipeline's input node
	input_node_idx: GraphNodeIdx,

	/// The index of this pipeline's output node
	output_node_idx: GraphNodeIdx,

	/// Map node names to node indices
	/// (used when connecting port-to-port outputs)
	node_output_name_map_ptp: RefCell<HashMap<PipelineNodeLabel, GraphNodeIdx>>,

	/// Map node names to node indices
	/// (used when connecting port-to-port inputs)
	node_input_name_map_ptp: RefCell<HashMap<PipelineNodeLabel, GraphNodeIdx>>,

	/// Map node names to node indices
	/// (used when connecting "after" outputs)
	node_output_name_map_after: RefCell<HashMap<PipelineNodeLabel, GraphNodeIdx>>,

	/// Map node names to node indices
	/// (used when connecting "after" inputs)
	node_input_name_map_after: RefCell<HashMap<PipelineNodeLabel, GraphNodeIdx>>,
}

impl<'a, StubType: PipelineNodeStub> PipelineBuilder<StubType> {
	pub fn build(
		context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		name: &str,
		spec: PipelineSpec<StubType>,
	) -> Result<Pipeline<StubType>, PipelinePrepareError<SDataStub<StubType>>> {
		debug!(
			source = "syntax",
			summary = "Building pipeline",
			name = name,
		);

		// Initialize all variables
		let builder = {
			let mut graph = Graph::new();

			// Add input and output nodes to the graph
			let input_node_idx = graph.add_node(("INPUT".into(), spec.input.clone()));
			let output_node_idx = graph.add_node(("OUTPUT".into(), spec.output.node_type.clone()));

			Self {
				name: name.into(),
				context,
				spec,
				graph: RefCell::new(graph),
				input_node_idx,
				output_node_idx,
				node_output_name_map_ptp: RefCell::new(HashMap::new()),
				node_input_name_map_ptp: RefCell::new(HashMap::new()),
				node_output_name_map_after: RefCell::new(HashMap::new()),
				node_input_name_map_after: RefCell::new(HashMap::new()),
			}
		};

		// Make sure every node's inputs are valid,
		// create the corresponding edges in the graph.
		debug!(source = "syntax", summary = "Checking inputs",);
		{
			for (node_label, node_spec) in &builder.spec.nodes {
				for (input_name, out_link) in &node_spec.inputs {
					builder.check_link(
						out_link,
						&NodeInput::Node {
							node: node_label.clone(),
							port: input_name.clone(),
						},
					)?;
				}
			}

			// Output node is handled separately
			for (input_name, out_link) in &builder.spec.output.inputs {
				builder.check_link(
					out_link,
					&NodeInput::Pipeline {
						port: input_name.clone(),
					},
				)?;
			}
		}

		debug!(source = "syntax", summary = "Making nodes",);
		// Add nodes to the graph
		for (node_name, node_spec) in builder.spec.nodes.iter() {
			// If this is a normal node, just add it.
			let n = builder
				.graph
				.borrow_mut()
				.add_node((node_name.clone(), node_spec.node_type.clone()));
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

		debug!(source = "syntax", summary = "Making after edges",);
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
						PipelineEdge::After,
					);
				} else {
					return Err(PipelinePrepareError::NoNodeAfter {
						node: after_name.clone(),
						caused_by_after_in: node_name.clone(),
					});
				}
			}
		}

		debug!(source = "syntax", summary = "Making ptp edges",);
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

			// Output node is handled separately
			for (port_label, node_output) in &builder.spec.output.inputs {
				let in_port = builder.get_input(
					&builder.spec.output.node_type,
					port_label,
					&"OUTPUT".into(),
				)?;
				builder.add_to_graph(in_port, builder.output_node_idx, node_output)?;
			}
		}

		debug!(source = "syntax", summary = "looking for cycles",);
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

	/// Is the port labeled `input_port_label` of a node with type `node_type`
	/// compatible with the given input?
	#[inline(always)]
	fn is_input_compatible(
		&self,
		node_type: &StubType,
		input_port_label: &PipelinePortLabel,
		input_type: SDataStub<StubType>,
		// Only used for errors
		node_label: &PipelineNodeLabel,
	) -> Result<bool, PipelinePrepareError<SDataStub<StubType>>> {
		Ok({
			let idx = node_type
				.input_with_name(&self.context, input_port_label)
				.ok_or(PipelinePrepareError::NoNodeInput {
					node: PipelineErrorNode::Named(node_label.clone()),
					input: input_port_label.clone(),
				})?;
			node_type.input_compatible_with(&self.context, idx, input_type)
		})
	}

	/// Find the port index of the input port labeled `input_port_label`
	/// of a node with type `node_type`.
	#[inline(always)]
	fn get_input(
		&self,
		node_type: &StubType,
		input_port_label: &PipelinePortLabel,

		// Only used for errors
		node_label: &PipelineNodeLabel,
	) -> Result<usize, PipelinePrepareError<SDataStub<StubType>>> {
		match node_type {
			t => t.input_with_name(&self.context, input_port_label),
		}
		.ok_or(PipelinePrepareError::NoNodeInput {
			node: PipelineErrorNode::Named(node_label.clone()),
			input: input_port_label.clone(),
		})
	}

	/// Find the port index and type of the output port labeled `output_port_label`
	/// of a node with type `node_type`.
	///
	/// This provides both `get_input` and `is_input_compatible` for outputs.
	/// Outputs are simpler than inputs---they always have exactly one type.
	#[inline(always)]
	fn get_output(
		&self,
		node_type: &StubType,
		output_port_label: &PipelinePortLabel,

		// Only used for errors
		node_label: &PipelineNodeLabel,
	) -> Result<(usize, SDataStub<StubType>), PipelinePrepareError<SDataStub<StubType>>> {
		match node_type {
			t => {
				let idx = t.output_with_name(&self.context, output_port_label).ok_or(
					PipelinePrepareError::NoNodeOutput {
						output: output_port_label.clone(),
						node: PipelineErrorNode::Named(node_label.clone()),
					},
				)?;
				let output_type = t.output_type(&self.context, idx);
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
		out_link: &NodeOutput<StubType>,
	) -> Result<(), PipelinePrepareError<SDataStub<StubType>>> {
		match out_link {
			NodeOutput::Pipeline { port } => {
				let out_port = self
					.spec
					.input
					.output_with_name(&self.context, port)
					.unwrap();
				self.graph.borrow_mut().add_edge(
					self.input_node_idx,
					node_idx,
					PipelineEdge::PortToPort((out_port, in_port)),
				);
			}
			NodeOutput::Inline(node) => {
				let x = self
					.graph
					.borrow_mut()
					.add_node(("INLINE".into(), node.clone()));
				self.graph.borrow_mut().add_edge(
					x,
					node_idx,
					PipelineEdge::PortToPort((0, in_port)),
				);
			}
			NodeOutput::Node { node, port } => {
				let out_port = self
					.get_output(&self.spec.nodes.get(node).unwrap().node_type, port, node)?
					.0;
				self.graph.borrow_mut().add_edge(
					*self.node_output_name_map_ptp.borrow().get(node).unwrap(),
					node_idx,
					PipelineEdge::PortToPort((out_port, in_port)),
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
		output: &NodeOutput<StubType>,
		input: &NodeInput,
	) -> Result<(), PipelinePrepareError<SDataStub<StubType>>> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type: SDataStub<StubType> = match output {
			NodeOutput::Inline(node) => {
				// Inline nodes must have exactly one output
				if node.n_outputs(&self.context) != 1 {
					return Err(PipelinePrepareError::BadInlineNode {
						input: input.clone(),
					});
				}
				node.output_type(&self.context, 0)
			}

			NodeOutput::Pipeline { port } => {
				if let Some(idx) = self.spec.input.output_with_name(&self.context, port) {
					self.spec.input.output_type(&self.context, idx)
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
			NodeInput::Pipeline { port } => {
				if let Some(idx) = self
					.spec
					.output
					.node_type
					.input_with_name(&self.context, port)
				{
					self.spec.output.node_type.input_compatible_with(
						&self.context,
						idx,
						output_type,
					)
				} else {
					return Err(PipelinePrepareError::NoNodeInput {
						node: PipelineErrorNode::PipelineOutput,
						input: port.clone(),
					});
				}
			}

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
					NodeOutput::Inline(_) => (PipelineErrorNode::Inline, "INLINE".into()),
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
