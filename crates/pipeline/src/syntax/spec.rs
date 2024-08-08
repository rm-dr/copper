//! A user-provided pipeline specification

use itertools::Itertools;
use petgraph::{algo::toposort, graph::Node, graphmap::GraphMap, Directed};
use serde::{de::DeserializeOwned, Deserialize};
use serde_with::{self, serde_as};
use smartstring::{LazyCompact, SmartString};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

use super::{
	errors::{PipelineErrorNode, PipelinePrepareError},
	labels::{PipelineNodeLabel, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
};
use crate::{
	api::{PipelineData, PipelineNode, PipelineNodeStub},
	graph::{graph::Graph, util::GraphNodeIdx},
	pipeline::{Pipeline, PipelineEdge},
	portspec::PipelinePortSpec,
};

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(crate) enum InternalNodeStub<StubType: PipelineNodeStub> {
	Pipeline {
		pipeline: String,
	},

	#[serde(untagged)]
	User(StubType),
}

impl<StubType: PipelineNodeStub> Debug for InternalNodeStub<StubType> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Pipeline { .. } => todo!(),
			Self::User(x) => x.fmt(f),
		}
	}
}

impl<StubType: PipelineNodeStub> PipelineNodeStub for InternalNodeStub<StubType> {
	type NodeType = StubType::NodeType;

	fn build(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
		name: &str,
	) -> Self::NodeType {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => StubType::build(n, ctx, name),
		}
	}

	fn inputs(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.inputs(ctx),
		}
	}

	fn outputs(
		&self,
		ctx: Arc<<Self::NodeType as PipelineNode>::NodeContext>,
	) -> PipelinePortSpec<<<Self::NodeType as PipelineNode>::DataType as PipelineData>::DataStub> {
		match self {
			Self::Pipeline { .. } => unreachable!(),
			Self::User(n) => n.outputs(ctx),
		}
	}
}

/// A description of a node in a pipeline
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(super) struct PipelineNodeSpec<StubType: PipelineNodeStub> {
	/// What kind of node is this?
	#[serde(rename = "node")]
	node_type: InternalNodeStub<StubType>,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde(rename = "input")]
	#[serde_as(as = "serde_with::Map<_, _>")]
	inputs: Vec<(PipelinePortLabel, NodeOutput<StubType>)>,

	#[serde(default)]
	/// Nodes that must complete before this node starts
	after: Vec<PipelineNodeLabel>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound = "StubType: DeserializeOwned")]
pub(crate) struct PipelineSpec<StubType: PipelineNodeStub> {
	/// This pipeline's input node.
	/// Note that this doesn't provide an `inputs` array.
	/// that is wired up by code that runs this pipeline.
	input: InternalNodeStub<StubType>,

	/// This pipeline's output node
	output: PipelineNodeSpec<StubType>,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	nodes: HashMap<PipelineNodeLabel, PipelineNodeSpec<StubType>>,
}

// TODO: warnings (disconnected input)
// TODO: check for unused nodes
impl<StubType: PipelineNodeStub> PipelineSpec<StubType> {
	#[inline(always)]
	fn get_input(
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,
		node_type: &InternalNodeStub<StubType>,

		node_name: &PipelineNodeLabel,
		input_name: &PipelinePortLabel,
	) -> Result<(usize,<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub), PipelinePrepareError<<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>>{
		match node_type {
			// `Pipeline` nodes don't know what inputs they provide,
			// we need to find them ourselves.
			InternalNodeStub::Pipeline { pipeline } => {
				let p = pipelines
					.iter()
					.find(|(x, _)| x == pipeline)
					.map(|(_, x)| x.clone())
					.ok_or(PipelinePrepareError::NoSuchPipeline {
						node: node_name.clone(),
						pipeline: pipeline.clone(),
					})?;
				p.graph
					.get_node(p.input_node_idx)
					.1
					.inputs(ctx.clone())
					.find_with_name(input_name)
			}
			t => t.inputs(ctx.clone()).find_with_name(input_name),
		}
		.ok_or(PipelinePrepareError::NoNodeInput {
			node: PipelineErrorNode::Named(node_name.clone()),
			input: input_name.clone(),
		})
	}

	#[inline(always)]
	fn get_output(
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,
		node_type: &InternalNodeStub<StubType>,

		node_name: &PipelineNodeLabel,
		output_name: &PipelinePortLabel,
	) -> Result<(usize,<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub), PipelinePrepareError<<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>>{
		match node_type {
			// `Pipeline` nodes don't know what inputs they provide,
			// we need to find them ourselves.
			InternalNodeStub::Pipeline { pipeline } => {
				let p = pipelines
					.iter()
					.find(|(x, _)| x == pipeline)
					.map(|(_, x)| x.clone())
					.ok_or(PipelinePrepareError::NoSuchPipeline {
						node: node_name.clone(),
						pipeline: pipeline.clone(),
					})?;
				p.graph
					.get_node(p.output_node_idx)
					.1
					.outputs(ctx.clone())
					.find_with_name(output_name)
			}
			t => t.outputs(ctx.clone()).find_with_name(output_name),
		}
		.ok_or(PipelinePrepareError::NoNodeOutput {
			output: output_name.clone(),
			node: PipelineErrorNode::Named(node_name.clone()),
		})
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
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,

		output: &NodeOutput<StubType>,
		input: &NodeInput,
	) -> Result<(), PipelinePrepareError<<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>>{
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type: <<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub = match output {
			NodeOutput::Inline(node) => {
				// Inline nodes must have exactly one output
				if node.outputs(ctx.clone()).len() != 1 {
					return Err(PipelinePrepareError::BadInlineNode { input: input.clone() })
				}
				node.outputs(ctx.clone()).iter().next().unwrap().1
			},

			NodeOutput::Pipeline { port } => {
				if let Some((_, from_type)) = self
					.input
					.outputs(ctx.clone())
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
				let get_node = self.nodes.get(node);

				if get_node.is_none() {
					return Err(PipelinePrepareError::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					});
				}
				let get_node = get_node.unwrap();
				Self::get_output(ctx.clone(), pipelines, &get_node.node_type, node, port)?.1
			}
		};

		// Find the datatype of the input port we're connecting to.
		// While doing this, make sure both the input node and port exist.
		let input_type = match &input {
			NodeInput::Pipeline { port } => {
				if let Some((_, from_type)) = self
					.output
					.node_type
					.inputs(ctx.clone())
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
				let get_node = self.nodes.get(node);

				if get_node.is_none() {
					return Err(PipelinePrepareError::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					});
				}
				let get_node = get_node.unwrap();
				Self::get_input(ctx, pipelines, &get_node.node_type, node, port)?.1
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

	/// Connect `out_link` to port index `in_port` of node `node_idx`.
	#[allow(clippy::too_many_arguments)]
	fn add_to_graph(
		&self,
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,

	// Current build state
		graph: &mut Graph<(PipelineNodeLabel, InternalNodeStub<StubType>), PipelineEdge>,
		node_output_name_map: &HashMap<PipelineNodeLabel, GraphNodeIdx>,
		input_node_idx: GraphNodeIdx,

		in_port: usize,
		node_idx: GraphNodeIdx,
		out_link: &NodeOutput<StubType>,
	) -> Result<(), PipelinePrepareError<<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>>{
		match out_link {
			NodeOutput::Pipeline { port } => {
				let out_port = self
					.input
					.outputs(ctx.clone())
					.iter()
					.enumerate()
					.find(|(_, (a, _))| a == port)
					.unwrap()
					.0;
				graph.add_edge(
					input_node_idx,
					node_idx,
					PipelineEdge::PortToPort((out_port, in_port)),
				);
			}
			NodeOutput::Inline(node) => {
				let x = graph.add_node(("INLINE".into(), node.clone()));
				graph.add_edge(x, node_idx, PipelineEdge::PortToPort((0, in_port)));
			}
			NodeOutput::Node { node, port } => {
				let out_port = Self::get_output(
					ctx,
					pipelines,
					&self.nodes.get(node).unwrap().node_type,
					node,
					port,
				)?
				.0;
				graph.add_edge(
					*node_output_name_map.get(node).unwrap(),
					node_idx,
					PipelineEdge::PortToPort((out_port, in_port)),
				);
			}
		}

		Ok(())
	}

	/// Check this pipeline spec's structure and use it to build a
	/// [`Pipeline`].
	pub fn prepare(
		self,
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline_name: String,
	// TODO: pipeline name type
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,
	) -> Result<Pipeline<StubType>, PipelinePrepareError<<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>>{
		let mut node_output_name_map: HashMap<PipelineNodeLabel, GraphNodeIdx> = HashMap::new();
		let mut node_input_name_map: HashMap<PipelineNodeLabel, GraphNodeIdx> = HashMap::new();

		let mut graph = Graph::new();
		let input_node_idx = graph.add_node(("INPUT".into(), self.input.clone()));
		let output_node_idx = graph.add_node(("OUTPUT".into(), self.output.node_type.clone()));

		for (node_name, node_spec) in &self.nodes {
			// Make sure all links going into this node are valid
			for (input_name, out_link) in &node_spec.inputs {
				self.check_link(
					ctx.clone(),
					pipelines,
					out_link,
					&NodeInput::Node {
						node: node_name.clone(),
						port: input_name.clone(),
					},
				)?;
			}

			// Add this node to our graph
			match &node_spec.node_type {
				// If this is a `Pipeline` node, add all nodes and edges inside the sub-pipeline
				InternalNodeStub::Pipeline { pipeline } => {
					let p = pipelines
						.iter()
						.find(|(x, _)| x == pipeline)
						.map(|(_, x)| x.clone())
						.ok_or(PipelinePrepareError::NoSuchPipeline {
							node: node_name.clone(),
							pipeline: pipeline.clone(),
						})?;

					let mut new_index_map = Vec::new();

					// Add other pipeline's nodes
					for (idx, (l, other_node)) in p.graph.iter_nodes_idx() {
						if idx == p.input_node_idx {
							let n = graph.add_node((
								format!("{}::{}", node_name, l).into(),
								other_node.clone(),
							));
							new_index_map.push(Some(n));
							node_input_name_map.insert(node_name.clone(), n);
						} else if idx == p.output_node_idx {
							let n = graph.add_node((
								format!("{}::{}", node_name, l).into(),
								other_node.clone(),
							));
							new_index_map.push(Some(n));
							node_output_name_map.insert(node_name.clone(), n);
						} else {
							// We intentionally don't insert to node_*_name_map here,
							// since we can't use the names of nodes in the inner
							// pipeline inside the outer pipeline
							new_index_map.push(Some(graph.add_node((
								format!("{}::{}", node_name, l).into(),
								other_node.clone(),
							))));
						}
					}

					// Add other pipeline's edges
					for (f, t, e) in p.graph.iter_edges() {
						graph.add_edge(
							new_index_map.get(f.as_usize()).unwrap().unwrap(),
							new_index_map.get(t.as_usize()).unwrap().unwrap(),
							e.clone(),
						);
					}
				}

				// If this is a normal node, just add it.
				_ => {
					let n = graph.add_node((node_name.clone(), node_spec.node_type.clone()));
					node_output_name_map.insert(node_name.clone(), n);
					node_input_name_map.insert(node_name.clone(), n);
				}
			}
		}

		// Make sure all "after" specifications are valid
		// and create their corresponding edges.
		for (node_name, node_spec) in &self.nodes {
			for after_name in node_spec.after.iter().unique() {
				if let Some(after_idx) = node_input_name_map.get(after_name) {
					graph.add_edge(
						*after_idx,
						*node_input_name_map.get(node_name).unwrap(),
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

		// Check final pipeline outputs
		for (out_name, out_link) in &self.output.inputs {
			self.check_link(
				ctx.clone(),
				pipelines,
				out_link,
				&NodeInput::Pipeline {
					port: out_name.clone(),
				},
			)?;
		}

		// Build graph
		for (node_name, node_spec) in &self.nodes {
			let node_idx = *node_input_name_map.get(node_name).unwrap();
			for (input_name, out_link) in node_spec.inputs.iter() {
				let in_port = Self::get_input(
					ctx.clone(),
					pipelines,
					&node_spec.node_type,
					node_name,
					input_name,
				)?
				.0;

				self.add_to_graph(
					ctx.clone(),
					pipelines,
					&mut graph,
					&node_output_name_map,
					input_node_idx,
					in_port,
					node_idx,
					out_link,
				)?;
			}
		}

		// Check for cycles
		// TODO: move to Graph module
		let mut fake_graph = GraphMap::<usize, (), Directed>::new();
		for (from, to, _) in graph.iter_edges() {
			// TODO: write custom cycle detection algorithm,
			// print all nodes that the cycle contains.
			// We don't need all edges---just node-to-node.
			fake_graph.add_edge((*from).into(), (*to).into(), ());
		}
		if toposort(&fake_graph, None).is_err() {
			return Err(PipelinePrepareError::HasCycle);
		}

		// Finish graph, adding output edges
		for (port_label, node_output) in &self.output.inputs {
			self.add_to_graph(
				ctx.clone(),
				pipelines,
				&mut graph,
				&node_output_name_map,
				input_node_idx,
				self.output
					.node_type
					.inputs(ctx.clone())
					.find_with_name(port_label)
					.unwrap()
					.0,
				output_node_idx,
				node_output,
			)?;
		}

		return Ok(Pipeline {
			name: pipeline_name.into(),
			graph: graph.finalize(),
			input_node_idx,
			output_node_idx,
		});
	}
}
