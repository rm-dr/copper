use std::{collections::HashMap, sync::Arc};

use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::Deserialize;

use crate::{
	data::{PipelineData, PipelineDataType},
	nodes::{PipelineNodeInstance, PipelineNodeType},
	pipeline::{NodePort, Pipeline},
	syntax::labels::PIPELINE_EXTERNAL_NODE_NAME,
};

use super::{
	labels::{PipelineNode, PipelineNodeLabel, PipelinePortLabel},
	ports::{NodeInput, NodeOutput},
	prepareresult::PipelinePrepareResult,
};

/// Pipeline configuration
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PipelineConfig {
	/// Names and types of pipeline inputs
	#[serde(default)]
	pub input: HashMap<PipelinePortLabel, PipelineDataType>,

	/// Names and types of pipeline outputs
	#[serde(default)]
	pub output: HashMap<PipelinePortLabel, PipelineDataType>,

	/// Map pipeline outputs to the node outputs that produce them
	#[serde(default)]
	pub outmap: HashMap<PipelinePortLabel, NodeOutput>,
}

/// A description of a node in a pipeline
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	pub node_type: PipelineNodeType,

	/// Where this node should read its input from.
	#[serde(default)]
	pub input: HashMap<PipelinePortLabel, NodeOutput>,
}

/// A description of a data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineSpec {
	/// Pipeline parameters
	pipeline: PipelineConfig,

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
	fn check_link(&self, output: &NodeOutput, input: &NodeInput) -> Option<PipelinePrepareResult> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type = match output {
			NodeOutput::InlineText { .. } => PipelineDataType::Text,

			NodeOutput::Node {
				node: PipelineNode::External,
				port,
			} => {
				if let Some(from_type) = self.pipeline.input.get(port) {
					*from_type
				} else {
					return Some(PipelinePrepareResult::NoNodeOutput {
						node: PipelineNode::External,
						output: port.clone(),
						caused_by: input.clone(),
					});
				}
			}

			NodeOutput::Node { node, port } => {
				let get_node = self.nodes.get(node.to_label_ref().unwrap());

				if get_node.is_none() {
					return Some(PipelinePrepareResult::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					});
				}
				let get_node = get_node.unwrap();
				let out_idx = get_node.node_type.output_with_name(port.into());

				if out_idx.is_none() {
					return Some(PipelinePrepareResult::NoNodeOutput {
						node: node.clone(),
						output: port.clone(),
						caused_by: input.clone(),
					});
				}
				get_node.node_type.output_type(out_idx.unwrap())
			}
		};

		// Find the datatype of the input port we're connecting to.
		// While doing this, make sure both the input node and port exist.
		let input_type = match &input {
			NodeInput::Node {
				node: PipelineNode::External,
				port,
			} => {
				if let Some(from_type) = self.pipeline.output.get(port) {
					*from_type
				} else {
					return Some(PipelinePrepareResult::NoNodeInput {
						node: PipelineNode::External,
						input: port.clone(),
					});
				}
			}

			NodeInput::Node { node, port } => {
				let get_node = self.nodes.get(node.to_label_ref().unwrap());

				if get_node.is_none() {
					return Some(PipelinePrepareResult::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					});
				}
				let get_node = get_node.unwrap();
				let in_idx = get_node.node_type.input_with_name(port.into());

				if in_idx.is_none() {
					return Some(PipelinePrepareResult::NoNodeInput {
						node: node.clone(),
						input: port.clone(),
					});
				}
				get_node.node_type.input_type(in_idx.unwrap())
			}
		};

		// Check types
		match output {
			NodeOutput::InlineText { .. } => {
				if input_type != PipelineDataType::Text {
					return Some(PipelinePrepareResult::InlineTypeMismatch {
						inline_type: PipelineDataType::Text,
						input: input.clone(),
					});
				}
			}
			NodeOutput::Node { .. } => {
				if output_type != input_type {
					return Some(PipelinePrepareResult::TypeMismatch {
						output: output.clone(),
						input: input.clone(),
					});
				}
			}
		};

		return None;
	}

	/// Connect `out_link` to port index `in_port` of node `node_idx`.
	fn add_to_graph(
		&self,
		// Current build state
		nodes: &mut Vec<PipelineNodeInstance>,
		edges: &mut Vec<(NodePort, NodePort)>,
		node_name_map: &HashMap<PipelineNode, usize>,

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
				nodes.push(PipelineNodeInstance::ConstantNode(Arc::new(
					PipelineData::Text(text.clone()),
				)));
			}
			NodeOutput::Node { node, port } => {
				let out_port = match node {
					PipelineNode::External => 0,
					PipelineNode::Node(x) => self
						.nodes
						.get(x)
						.unwrap()
						.node_type
						.output_with_name(port.into())
						.unwrap(),
				};
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

	pub fn prepare(&mut self) -> PipelinePrepareResult {
		// Ordered lists of pipeline inputs & outputs
		let pipeline_inputs = self.pipeline.input.keys().cloned().collect::<Vec<_>>();
		let pipeline_outputs = self.pipeline.output.keys().cloned().collect::<Vec<_>>();

		// Check each node's name and inputs;
		// Build node array and initialize external node;
		// Initialize nodes in graph
		let mut nodes = Vec::new();
		let mut edges = Vec::new();
		let mut node_name_map: HashMap<PipelineNode, usize> = HashMap::new();
		nodes.push(PipelineNodeInstance::ExternalNode);
		node_name_map.insert(PipelineNode::External, 0);
		for (node_name, node_spec) in &self.nodes {
			// Make sure we're not using a reserved name
			let s: &str = node_name.into();
			if s == PIPELINE_EXTERNAL_NODE_NAME {
				return PipelinePrepareResult::NodeHasReservedName {
					node: node_name.into(),
				};
			}

			for (input_name, out_link) in &node_spec.input {
				if let Some(x) = self.check_link(
					out_link,
					&NodeInput::Node {
						node: node_name.into(),
						port: input_name.clone(),
					},
				) {
					return x;
				};
			}

			node_name_map.insert(node_name.into(), nodes.len());
			nodes.push(node_spec.node_type.build(node_name.into()));
		}

		// Check final pipeline outputs
		for (out_name, out_link) in &self.pipeline.outmap {
			if let Some(x) = self.check_link(
				out_link,
				&NodeInput::Node {
					node: PipelineNode::External,
					port: out_name.clone(),
				},
			) {
				return x;
			};
		}

		// Build graph
		for (node_name, node_spec) in &self.nodes {
			let node_idx = *node_name_map.get(&node_name.into()).unwrap();
			for (input_name, out_link) in node_spec.input.iter() {
				let in_port = node_spec
					.node_type
					.input_with_name(input_name.into())
					.unwrap();

				self.add_to_graph(
					&mut nodes,
					&mut edges,
					&node_name_map,
					in_port,
					node_idx,
					out_link,
				);
			}
		}

		// Check for cycles
		let mut graph = GraphMap::<usize, (), Directed>::new();
		for (out_np, in_np) in edges.iter() {
			graph.add_edge(out_np.node_idx, in_np.node_idx, ());
		}

		// Finish graph, adding output edges
		for (port_label, node_output) in &self.pipeline.outmap {
			self.add_to_graph(
				&mut nodes,
				&mut edges,
				&node_name_map,
				pipeline_outputs
					.iter()
					.enumerate()
					.find(|(_, x)| *x == port_label)
					.unwrap()
					.0,
				*node_name_map.get(&PipelineNode::External).unwrap(),
				node_output,
			)
		}

		if toposort(&graph, None).is_err() {
			return PipelinePrepareResult::HasCycle;
		}

		return PipelinePrepareResult::Ok(Pipeline::new(
			nodes,
			edges,
			*node_name_map.get(&PipelineNode::External).unwrap(),
			pipeline_inputs,
			pipeline_outputs,
		));
	}
}
