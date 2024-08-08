//! A user-provided pipeline specification

use std::{collections::HashMap, sync::Arc};

use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::Deserialize;
use serde_with::{self, serde_as};
use ufo_util::data::{PipelineData, PipelineDataType};

use crate::{
	input::PipelineInputKind,
	nodes::nodetype::PipelineNodeType,
	output::PipelineOutputKind,
	pipeline::{NodePort, Pipeline},
};

use super::{
	errors::{PipelineErrorNode, PipelinePrepareError},
	labels::{PipelineNodeLabel, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
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
	#[serde(rename = "type")]
	node_type: PipelineNodeType,

	/// Where this node should read its input from.
	#[serde(default)]
	#[serde_as(as = "serde_with::Map<_, _>")]
	input: Vec<(PipelinePortLabel, NodeOutput)>,
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
						node: PipelineErrorNode::Pipeline,
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
				let a = get_node.node_type.outputs();
				let b = a.find_with_name(port);
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
						node: PipelineErrorNode::Pipeline,
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
				let a = get_node.node_type.inputs();
				let b = a.find_with_name(port);

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
		nodes: &mut Vec<(PipelineNodeLabel, PipelineNodeType)>,
		edges: &mut Vec<(NodePort, NodePort)>,
		node_name_map: &HashMap<PipelineNodeLabel, usize>,
		pipeline_inputs_idx: usize,

		in_port: usize,
		node_idx: usize,
		out_link: &NodeOutput,
	) {
		match out_link {
			NodeOutput::InlineText { text } => {
				edges.push((
					NodePort {
						// This must be done BEFORE pushing
						// to nodes.
						node_idx: nodes.len(),
						port: 0,
					},
					NodePort {
						node_idx,
						port: in_port,
					},
				));
				nodes.push((
					"".into(),
					PipelineNodeType::ConstantNode {
						value: Arc::new(PipelineData::Text(text.clone())),
					},
				));
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
				edges.push((
					NodePort {
						node_idx: pipeline_inputs_idx,
						port: out_port,
					},
					NodePort {
						node_idx,
						port: in_port,
					},
				));
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
				edges.push((
					NodePort {
						node_idx: *node_name_map.get(node).unwrap(),
						port: out_port,
					},
					NodePort {
						node_idx,
						port: in_port,
					},
				));
			}
		}
	}

	/// Check this pipeline spec's structure and use it to build a
	/// [`Pipeline`].
	pub fn prepare(self) -> Result<Pipeline, PipelinePrepareError> {
		// Check each node's name and inputs;
		// Build node array and initialize external node;
		// Initialize nodes in graph
		let mut nodes: Vec<(PipelineNodeLabel, PipelineNodeType)> = Vec::new();
		let mut edges = Vec::new();
		let mut node_name_map: HashMap<PipelineNodeLabel, usize> = HashMap::new();

		nodes.push((
			"".into(),
			PipelineNodeType::PipelineInputs {
				outputs: self.config.input.get_outputs().to_vec(),
			},
		));
		let pipeline_inputs_idx = 0;

		nodes.push((
			"".into(),
			PipelineNodeType::PipelineOutputs {
				inputs: self.config.output.get_inputs().to_vec(),
			},
		));
		let pipeline_outputs_idx = 1;

		for (node_name, node_spec) in &self.nodes {
			for (input_name, out_link) in &node_spec.input {
				self.check_link(
					out_link,
					&NodeInput::Node {
						node: node_name.clone(),
						port: input_name.clone(),
					},
				)?;
			}

			node_name_map.insert(node_name.clone(), nodes.len());
			nodes.push((node_name.clone(), node_spec.node_type.clone()));
		}

		// Check final pipeline outputs
		for (out_name, out_link) in &self.config.output_map {
			self.check_link(
				out_link,
				&NodeInput::Pipeline {
					port: out_name.clone(),
				},
			)?;
		}

		// Build graph
		for (node_name, node_spec) in &self.nodes {
			let node_idx = *node_name_map.get(node_name).unwrap();
			for (input_name, out_link) in node_spec.input.iter() {
				let in_port = node_spec
					.node_type
					.inputs()
					.find_with_name(input_name)
					.unwrap()
					.0;

				self.add_to_graph(
					&mut nodes,
					&mut edges,
					&node_name_map,
					pipeline_inputs_idx,
					in_port,
					node_idx,
					out_link,
				);
			}
		}

		// Build graph and check for cycles
		let mut graph = GraphMap::<usize, (), Directed>::new();
		for (out_np, in_np) in edges.iter() {
			graph.add_edge(out_np.node_idx, in_np.node_idx, ());
		}
		if toposort(&graph, None).is_err() {
			return Err(PipelinePrepareError::HasCycle);
		}

		// Finish graph, adding output edges
		for (port_label, node_output) in &self.config.output_map {
			self.add_to_graph(
				&mut nodes,
				&mut edges,
				&node_name_map,
				pipeline_inputs_idx,
				self.config
					.output
					.get_inputs()
					.find_with_name(port_label)
					.unwrap()
					.0,
				pipeline_outputs_idx,
				node_output,
			)
		}

		// Build edge maps
		let mut edge_map = (0..nodes.len()).map(|_| Vec::new()).collect::<Vec<_>>();
		let mut rev_edge_map = (0..nodes.len()).map(|_| Vec::new()).collect::<Vec<_>>();
		for (i, x) in edges.iter().enumerate() {
			edge_map[x.0.node_idx].push(i);
			rev_edge_map[x.1.node_idx].push(i);
		}

		return Ok(Pipeline {
			nodes,
			edges,

			edge_map_out: edge_map,
			edge_map_in: rev_edge_map,
			config: self.config.clone(),
		});
	}
}
