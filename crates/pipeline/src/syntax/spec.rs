//! A user-provided pipeline specification

use itertools::Itertools;
use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::Deserialize;
use serde_with::{self, serde_as};
use smartstring::{LazyCompact, SmartString};
use std::{collections::HashMap, sync::Arc};
use ufo_util::{
	data::{PipelineData, PipelineDataType},
	graph::{Graph, GraphNodeIdx},
};

use super::{
	errors::{PipelineErrorNode, PipelinePrepareError},
	labels::{PipelineNodeLabel, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
};
use crate::{
	input::PipelineInputKind,
	nodes::nodetype::PipelineNodeType,
	output::PipelineOutputKind,
	pipeline::{Pipeline, PipelineEdge},
};

/// Pipeline configuration
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
	/// The kind of input this pipeline takes
	pub input: PipelineInputKind,

	/// The kind of output this pipeline produces
	pub output: PipelineOutputKind,

	/// Connect node outputs to this pipeline's outputs
	#[serde(default)]
	#[serde_as(as = "serde_with::Map<_, _>")]
	pub output_map: Vec<(PipelinePortLabel, NodeOutput)>,
}

/// A description of a node in a pipeline
#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "node")]
	node_type: PipelineNodeType,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde_as(as = "serde_with::Map<_, _>")]
	input: Vec<(PipelinePortLabel, NodeOutput)>,

	#[serde(default)]
	/// Nodes that must complete before this node starts
	after: Vec<PipelineNodeLabel>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineSpec {
	/// Pipeline parameters
	config: PipelineConfig,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	nodes: HashMap<PipelineNodeLabel, PipelineNodeSpec>,
}

// TODO: warnings (disconnected input)
// TODO: check for unused nodes
impl PipelineSpec {
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
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline>)>,

		output: &NodeOutput,
		input: &NodeInput,
	) -> Result<(), PipelinePrepareError> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type = match output {
			NodeOutput::InlineText { .. } => PipelineDataType::Text,

			NodeOutput::Pipeline { port } => {
				if let Some((_, from_type)) = self
					.config
					.input
					.get_outputs()
					.iter()
					.find(|(a, _)| a == port)
				{
					from_type
				} else {
					return Err(PipelinePrepareError::NoNodeOutput {
						node: PipelineErrorNode::PipelineInput,
						output: port.clone(),
						caused_by: input.clone(),
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
				// TODO: this should be a function
				let b = match &get_node.node_type {
					PipelineNodeType::Pipeline { pipeline } => {
						// `Pipeline` nodes don't know what outputs they provide,
						// we need to find them ourselves.
						let p = pipelines
							.iter()
							.find(|(x, _)| x == pipeline)
							.map(|(_, x)| x.clone())
							.ok_or(PipelinePrepareError::NoSuchPipeline {
								node: node.clone(),
								pipeline: pipeline.clone(),
							})?;
						// Note that we're looking at the *inputs* of `p`'s pipeline *output*.
						// Inputs inside of p become outputs in this pipeline.
						p.config.output.get_inputs().find_with_name(port)
					}
					t => t.outputs().find_with_name(port),
				};

				if b.is_none() {
					return Err(PipelinePrepareError::NoNodeOutput {
						node: PipelineErrorNode::Named(node.clone()),
						output: port.clone(),
						caused_by: input.clone(),
					});
				}
				b.unwrap().1
			}
		};

		// Find the datatype of the input port we're connecting to.
		// While doing this, make sure both the input node and port exist.
		let input_type = match &input {
			NodeInput::Pipeline { port } => {
				if let Some((_, from_type)) = self.config.output.get_inputs().find_with_name(port) {
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
				let b = match &get_node.node_type {
					// `Pipeline` nodes don't know what inputs they provide,
					// we need to find them ourselves.
					PipelineNodeType::Pipeline { pipeline } => {
						let p = pipelines
							.iter()
							.find(|(x, _)| x == pipeline)
							.map(|(_, x)| x.clone())
							.ok_or(PipelinePrepareError::NoSuchPipeline {
								node: node.clone(),
								pipeline: pipeline.clone(),
							})?;
						// Note that we're looking at the *outputs* of `p`'s pipeline *input*.
						// Outputs inside of p become inputs in this pipeline.
						p.config.input.get_outputs().find_with_name(port)
					}
					t => t.inputs().find_with_name(port),
				};

				if b.is_none() {
					return Err(PipelinePrepareError::NoNodeInput {
						node: PipelineErrorNode::Named(node.clone()),
						input: port.clone(),
					});
				}
				b.unwrap().1
			}
		};

		// Check types
		match output {
			NodeOutput::InlineText { .. } => {
				if input_type != PipelineDataType::Text {
					return Err(PipelinePrepareError::InlineTypeMismatch {
						inline_type: PipelineDataType::Text,
						input: input.clone(),
					});
				}
			}
			NodeOutput::Pipeline { .. } | NodeOutput::Node { .. } => {
				if output_type != input_type {
					return Err(PipelinePrepareError::TypeMismatch {
						output: output.clone(),
						input: input.clone(),
					});
				}
			}
		};

		return Ok(());
	}

	/// Connect `out_link` to port index `in_port` of node `node_idx`.
	#[allow(clippy::too_many_arguments)]
	fn add_to_graph(
		&self,
		// Current build state
		graph: &mut Graph<(PipelineNodeLabel, PipelineNodeType), PipelineEdge>,
		node_output_name_map: &HashMap<PipelineNodeLabel, GraphNodeIdx>,
		input_node_idx: GraphNodeIdx,

		in_port: usize,
		node_idx: GraphNodeIdx,
		out_link: &NodeOutput,
	) {
		match out_link {
			NodeOutput::InlineText { text } => {
				let n = graph.add_node((
					"CONSTANT".into(),
					PipelineNodeType::ConstantNode {
						value: PipelineData::Text(Arc::new(text.clone())),
					},
				));
				graph.add_edge(n, node_idx, PipelineEdge::PortToPort((0, in_port)));
			}
			NodeOutput::Pipeline { port } => {
				let out_port = self
					.config
					.input
					.get_outputs()
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
			NodeOutput::Node { node, port } => {
				let out_port = self
					.nodes
					.get(node)
					.unwrap()
					.node_type
					.outputs()
					.find_with_name(port)
					.unwrap()
					.0;
				graph.add_edge(
					*node_output_name_map.get(node).unwrap(),
					node_idx,
					PipelineEdge::PortToPort((out_port, in_port)),
				);
			}
		}
	}

	/// Check this pipeline spec's structure and use it to build a
	/// [`Pipeline`].
	pub fn prepare(
		self,
		pipeline_name: String,
		// TODO: pipeline name type
		pipelines: &Vec<(SmartString<LazyCompact>, Arc<Pipeline>)>,
	) -> Result<Pipeline, PipelinePrepareError> {
		let mut node_output_name_map: HashMap<PipelineNodeLabel, GraphNodeIdx> = HashMap::new();
		let mut node_input_name_map: HashMap<PipelineNodeLabel, GraphNodeIdx> = HashMap::new();

		let mut graph = Graph::new();

		let input_node_idx = graph.add_node((
			"INPUT".into(),
			PipelineNodeType::PipelineInputs {
				input_type: self.config.input.clone(),
				outputs: self.config.input.get_outputs().to_vec(),
			},
		));

		let output_node_idx = graph.add_node((
			"OUTPUT".into(),
			PipelineNodeType::PipelineOutputs {
				output_type: self.config.output.clone(),
				pipeline: pipeline_name.clone().into(),
				inputs: self.config.output.get_inputs().to_vec(),
			},
		));

		for (node_name, node_spec) in &self.nodes {
			// Make sure all links going into this node are valid
			for (input_name, out_link) in &node_spec.input {
				self.check_link(
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
				PipelineNodeType::Pipeline { pipeline } => {
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
					for (l, other_node) in p.graph.iter_nodes() {
						match other_node {
							PipelineNodeType::PipelineInputs { input_type, .. } => {
								let n = graph.add_node((
									format!("{}::{}", node_name, l).into(),
									input_type.to_node_type().unwrap(),
								));
								new_index_map.push(Some(n));
								node_input_name_map.insert(node_name.clone(), n);
							}
							PipelineNodeType::PipelineOutputs { .. } => {
								let n = graph.add_node((
									format!("{}::{}", node_name, l).into(),
									other_node.clone(),
								));
								new_index_map.push(Some(n));
								node_output_name_map.insert(node_name.clone(), n);
							}
							_ => {
								// We intentionally don't insert to node_*_name_map here,
								// since we can't use the names of nodes in the inner
								// pipeline inside the outer pipeline
								new_index_map.push(Some(graph.add_node((
									format!("{}::{}", node_name, l).into(),
									other_node.clone(),
								))));
							}
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
		for (out_name, out_link) in &self.config.output_map {
			self.check_link(
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
			for (input_name, out_link) in node_spec.input.iter() {
				let in_port = match &node_spec.node_type {
					// `Pipeline` nodes don't know what inputs they provide,
					// we need to find them ourselves.
					PipelineNodeType::Pipeline { pipeline } => {
						let p = pipelines
							.iter()
							.find(|(x, _)| x == pipeline)
							.map(|(_, x)| x.clone())
							.ok_or(PipelinePrepareError::NoSuchPipeline {
								node: node_name.clone(),
								pipeline: pipeline.clone(),
							})?;
						// Note that we're looking at the *outputs* of `p`'s pipeline *input*.
						// Outputs inside of p become inputs in this pipeline.
						p.config.input.get_outputs().find_with_name(input_name)
					}
					t => t.inputs().find_with_name(input_name),
				}
				.unwrap()
				.0;

				self.add_to_graph(
					&mut graph,
					&node_output_name_map,
					input_node_idx,
					in_port,
					node_idx,
					out_link,
				);
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
		for (port_label, node_output) in &self.config.output_map {
			self.add_to_graph(
				&mut graph,
				&node_output_name_map,
				input_node_idx,
				self.config
					.output
					.get_inputs()
					.find_with_name(port_label)
					.unwrap()
					.0,
				output_node_idx,
				node_output,
			)
		}

		return Ok(Pipeline {
			name: pipeline_name.into(),
			graph: graph.finalize(),
			config: self.config.clone(),
		});
	}
}
