//! A user-provided pipeline specification

use itertools::Itertools;
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use super::{
	errors::{PipelineErrorNode, PipelinePrepareError},
	internalnode::InternalNodeStub,
	ports::{NodeInput, NodeOutput},
	spec::PipelineSpec,
};
use crate::{
	api::{PipelineNode, PipelineNodeStub},
	graph::{graph::Graph, util::GraphNodeIdx},
	labels::{PipelineLabel, PipelineNodeLabel, PipelinePortLabel},
	pipeline::{Pipeline, PipelineEdge},
	SDataStub,
};

pub(crate) struct PipelineBuilder<'a, StubType: PipelineNodeStub> {
	/// The name of the pipeline we're building
	name: PipelineLabel,

	/// The context with which to build this pipeline
	context: <StubType::NodeType as PipelineNode>::NodeContext,

	/// The pipeline spec to build
	spec: PipelineSpec<StubType>,

	/// Other pipelines we've already built
	pipelines: &'a Vec<Arc<Pipeline<StubType>>>,

	/// The pipeline graph we're building
	graph: RefCell<Graph<(PipelineNodeLabel, InternalNodeStub<StubType>), PipelineEdge>>,

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

impl<'a, StubType: PipelineNodeStub> PipelineBuilder<'a, StubType> {
	pub fn build(
		context: <StubType::NodeType as PipelineNode>::NodeContext,
		pipelines: &'a Vec<Arc<Pipeline<StubType>>>,
		name: &str,
		spec: PipelineSpec<StubType>,
	) -> Result<Pipeline<StubType>, PipelinePrepareError<SDataStub<StubType>>> {
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
				pipelines,
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

		// Add nodes to the graph
		for (node_name, node_spec) in builder.spec.nodes.iter() {
			match &node_spec.node_type {
				InternalNodeStub::Pipeline { pipeline } => {
					// If this is a `Pipeline` node, add the pipeline's contents
					builder.add_pipeline(node_name, pipeline)?;
				}

				_ => {
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
			}
		}

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

		// Make sure all "port to port" edges are valid and create them in the graph.
		{
			for (node_name, node_spec) in &builder.spec.nodes {
				let node_idx = *builder
					.node_input_name_map_ptp
					.borrow()
					.get(node_name)
					.unwrap();
				for (input_name, out_link) in node_spec.inputs.iter() {
					let in_port = builder
						.get_input(&node_spec.node_type, input_name, node_name)?
						.0;
					builder.add_to_graph(in_port, node_idx, out_link)?;
				}
			}

			// Output node is handled separately
			for (port_label, node_output) in &builder.spec.output.inputs {
				let in_port = builder
					.get_input(&builder.spec.output.node_type, port_label, &"OUTPUT".into())?
					.0;
				builder.add_to_graph(in_port, builder.output_node_idx, node_output)?;
			}
		}

		// Make sure our graph doesn't have any cycles
		if builder.graph.borrow().has_cycle() {
			return Err(PipelinePrepareError::HasCycle);
		}

		return Ok(Pipeline {
			name: builder.name,
			graph: builder.graph.into_inner().finalize(),
			input_node_idx: builder.input_node_idx,
			output_node_idx: builder.output_node_idx,
		});
	}

	/// Find the port index and type of the input port labeled `input_port label`
	/// of a node with type `node_type`.
	#[inline(always)]
	fn get_input(
		&self,
		node_type: &InternalNodeStub<StubType>,
		input_port_label: &PipelinePortLabel,

		// Only used for errors
		node_label: &PipelineNodeLabel,
	) -> Result<(usize, SDataStub<StubType>), PipelinePrepareError<SDataStub<StubType>>> {
		match node_type {
			// `Pipeline` nodes don't know what inputs they provide,
			// we need to find them ourselves.
			InternalNodeStub::Pipeline { pipeline } => {
				let p = self
					.pipelines
					.iter()
					.find(|x| x.name == *pipeline)
					.cloned()
					.ok_or(PipelinePrepareError::NoSuchPipeline {
						node: node_label.clone(),
						pipeline: pipeline.clone(),
					})?;
				p.graph
					.get_node(p.input_node_idx)
					.1
					.inputs(&self.context)
					.find_with_name(input_port_label)
			}
			t => t.inputs(&self.context).find_with_name(input_port_label),
		}
		.ok_or(PipelinePrepareError::NoNodeInput {
			node: PipelineErrorNode::Named(node_label.clone()),
			input: input_port_label.clone(),
		})
	}

	/// Find the port index and type of the output port labeled `output_port_label`
	/// of a node with type `node_type`.
	#[inline(always)]
	fn get_output(
		&self,
		node_type: &InternalNodeStub<StubType>,
		output_port_label: &PipelinePortLabel,

		// Only used for errors
		node_label: &PipelineNodeLabel,
	) -> Result<(usize, SDataStub<StubType>), PipelinePrepareError<SDataStub<StubType>>> {
		match node_type {
			// `Pipeline` nodes don't know what inputs they provide,
			// we need to find them ourselves.
			InternalNodeStub::Pipeline { pipeline } => {
				let p = self
					.pipelines
					.iter()
					.find(|x| x.name == *pipeline)
					.cloned()
					.ok_or(PipelinePrepareError::NoSuchPipeline {
						node: node_label.clone(),
						pipeline: pipeline.clone(),
					})?;
				p.graph
					.get_node(p.output_node_idx)
					.1
					.outputs(&self.context)
					.find_with_name(output_port_label)
			}
			t => t.outputs(&self.context).find_with_name(output_port_label),
		}
		.ok_or(PipelinePrepareError::NoNodeOutput {
			output: output_port_label.clone(),
			node: PipelineErrorNode::Named(node_label.clone()),
		})
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
					.outputs(&self.context)
					.iter()
					.enumerate()
					.find(|(_, (a, _))| a == port)
					.unwrap()
					.0;
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
				if node.outputs(&self.context).len() != 1 {
					return Err(PipelinePrepareError::BadInlineNode {
						input: input.clone(),
					});
				}
				node.outputs(&self.context).iter().next().unwrap().1
			}

			NodeOutput::Pipeline { port } => {
				if let Some((_, from_type)) = self
					.spec
					.input
					.outputs(&self.context)
					.iter()
					.find(|(a, _)| a == port)
				{
					from_type
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
		let input_type = match &input {
			NodeInput::Pipeline { port } => {
				if let Some((_, from_type)) = self
					.spec
					.output
					.node_type
					.inputs(&self.context)
					.find_with_name(port)
				{
					from_type
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
				self.get_input(&get_node.node_type, port, node)?.1
			}
		};

		// Check types
		if output_type != input_type {
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
				input_type,
			});
		}

		return Ok(());
	}

	/// Add a sub-pipeline to this graph.
	///
	/// Replaces the node named `node_name` with the contents of
	/// the pipeline named `pipeline_name`.
	///
	/// The node being replaced must always be an [`InternalNodeStub::Pipeline`].
	fn add_pipeline(
		&self,
		node_name: &PipelineNodeLabel,
		pipeline_name: &PipelineLabel,
	) -> Result<(), PipelinePrepareError<SDataStub<StubType>>> {
		let p = self
			.pipelines
			.iter()
			.find(|x| x.name == *pipeline_name)
			.cloned()
			.ok_or(PipelinePrepareError::NoSuchPipeline {
				node: node_name.clone(),
				pipeline: pipeline_name.clone(),
			})?;

		let mut new_index_map = Vec::new();

		// Add other pipeline's nodes
		for (idx, (l, other_node)) in p.graph.iter_nodes_idx() {
			if idx == p.input_node_idx {
				let n = self
					.graph
					.borrow_mut()
					.add_node((format!("{}::{}", node_name, l).into(), other_node.clone()));
				new_index_map.push(Some(n));

				// This is why we have different name maps for "ptp" and "after" nodes.
				// ptp nodes that end at a pipeline node should be remapped to that pipeline's INPUT.
				// "after" nodes that end at a pipeline node should be remapped to that pipeline's OUTPUT.
				// (so that "after" waits for the whole sub-pipeline to finish)
				//
				// Similarly, "after" nodes that START at a pipeline node should be moved to start at
				// the pipeline's INPUT node, so that the whole pipeline must wait.
				self.node_input_name_map_ptp
					.borrow_mut()
					.insert(node_name.clone(), n);
				self.node_output_name_map_after
					.borrow_mut()
					.insert(node_name.clone(), n);
			} else if idx == p.output_node_idx {
				let n = self
					.graph
					.borrow_mut()
					.add_node((format!("{}::{}", node_name, l).into(), other_node.clone()));
				new_index_map.push(Some(n));

				self.node_output_name_map_ptp
					.borrow_mut()
					.insert(node_name.clone(), n);
				self.node_input_name_map_after
					.borrow_mut()
					.insert(node_name.clone(), n);
			} else {
				// We intentionally don't insert to node_*_name_map here,
				// since we can't use the names of nodes in the inner
				// pipeline inside the outer pipeline
				new_index_map.push(Some(
					self.graph
						.borrow_mut()
						.add_node((format!("{}::{}", node_name, l).into(), other_node.clone())),
				));
			}
		}

		// Add other pipeline's edges
		for (f, t, e) in p.graph.iter_edges() {
			self.graph.borrow_mut().add_edge(
				new_index_map.get(f.as_usize()).unwrap().unwrap(),
				new_index_map.get(t.as_usize()).unwrap().unwrap(),
				e.clone(),
			);
		}

		return Ok(());
	}
}
