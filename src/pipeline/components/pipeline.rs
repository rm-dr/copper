use std::collections::HashMap;

use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::Deserialize;

use super::{
	checkresult::PipelineCheckResult,
	labels::{PipelineNode, PipelineNodeLabel, PipelinePortLabel, PIPELINE_NODE_NAME},
	ports::{NodeInput, NodeOutput},
};
use crate::pipeline::{
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
	nodes::PipelineNodeTypes,
};

/// Pipeline configuration
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
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

/// A pipeline node specification
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	pub node_type: PipelineNodeTypes,

	/// Where this node should read its input from.
	#[serde(default)]
	pub input: HashMap<PipelinePortLabel, NodeOutput>,
}

#[derive(Debug)]
enum PipelineCheckState {
	Unchecked,
	Failed,
	Passed,
}

impl Default for PipelineCheckState {
	fn default() -> Self {
		Self::Unchecked
	}
}

/// A data processing pipeline
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pipeline {
	/// Pipeline parameters
	pipeline: PipelineConfig,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	nodes: HashMap<PipelineNodeLabel, PipelineNodeSpec>,

	/// Has this pipeline passed [`Pipeline::check()`]?
	#[serde(skip)]
	check_state: PipelineCheckState,
}

// TODO: pretty errors
// TODO: warnings (disconnected input)
// TODO: check for unused nodes
impl Pipeline {
	/// Check a link from `output` to `input`.
	/// Returns [`PipelineCheckResult::Ok`] if everything is ok, and an error otherwise.
	///
	/// This makes sure that...
	/// - The output node exists
	/// - The output node has the specified port
	/// - The input node exists
	/// - The input node has the specified port
	/// - The input and output ports have matching types
	fn check_link(&self, output: &NodeOutput, input: &NodeInput) -> PipelineCheckResult {
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
					return PipelineCheckResult::NoNodeOutput {
						node: PipelineNode::External,
						output: port.clone(),
						caused_by: input.clone(),
					};
				}
			}

			NodeOutput::Node { node, port } => {
				let get_node = self.nodes.get(node.to_label_ref().unwrap());

				if get_node.is_none() {
					return PipelineCheckResult::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					};
				}
				let get_node = get_node.unwrap();
				let input_spec = get_node.node_type.get_output(port);

				if input_spec.is_none() {
					return PipelineCheckResult::NoNodeOutput {
						node: node.clone(),
						output: port.clone(),
						caused_by: input.clone(),
					};
				}
				input_spec.unwrap()
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
					return PipelineCheckResult::NoNodeInput {
						node: PipelineNode::External,
						input: port.clone(),
					};
				}
			}

			NodeInput::Node { node, port } => {
				let get_node = self.nodes.get(node.to_label_ref().unwrap());

				if get_node.is_none() {
					return PipelineCheckResult::NoNode {
						node: node.clone(),
						caused_by: input.clone(),
					};
				}
				let get_node = get_node.unwrap();
				let input = get_node.node_type.get_input(port);

				if input.is_none() {
					return PipelineCheckResult::NoNodeInput {
						node: node.clone(),
						input: port.clone(),
					};
				}
				input.unwrap()
			}
		};

		// Check types
		match output {
			NodeOutput::InlineText { .. } => {
				if input_type != PipelineDataType::Text {
					return PipelineCheckResult::InlineTypeMismatch {
						inline_type: PipelineDataType::Text,
						input: input.clone(),
					};
				}
			}
			NodeOutput::Node { .. } => {
				if output_type != input_type {
					return PipelineCheckResult::TypeMismatch {
						output: output.clone(),
						input: input.clone(),
					};
				}
			}
		};

		return PipelineCheckResult::Ok;
	}

	/// Build a graph using this pipeline's specs.
	///
	/// This assumes that most of [`Pipeline::check()`] has passed,
	/// except for the cycle check (because this method is used to run that check).
	fn build_graph(&self) -> GraphMap<&str, (), Directed> {
		// We don't need to create nodes explicitly,
		// since `add_edge` does this automatically.
		let mut graph = GraphMap::<&str, (), Directed>::new();
		for (node_name, node_spec) in &self.nodes {
			for out_link in node_spec.input.values() {
				match out_link {
					NodeOutput::InlineText { .. } => {}
					NodeOutput::Node { node, .. } => {
						graph.add_edge(node.into(), node_name.into(), ());
					}
				}
			}
		}

		return graph;
	}

	pub fn check(&mut self) -> PipelineCheckResult {
		self.check_state = PipelineCheckState::Failed;

		// Check each node's name and inputs
		for (node_name, node_spec) in &self.nodes {
			// Make sure we're not using a reserved name
			let s: &str = node_name.into();
			if s == PIPELINE_NODE_NAME {
				return PipelineCheckResult::NodeHasReservedName {
					node: node_name.into(),
				};
			}

			for (input_name, out_link) in &node_spec.input {
				match self.check_link(
					out_link,
					&NodeInput::Node {
						node: node_name.into(),
						port: input_name.clone(),
					},
				) {
					PipelineCheckResult::Ok => {}
					x => return x,
				};
			}
		}

		// Check final pipeline outputs
		for (out_name, out_link) in &self.pipeline.outmap {
			match self.check_link(
				out_link,
				&NodeInput::Node {
					node: PipelineNode::External,
					port: out_name.clone(),
				},
			) {
				PipelineCheckResult::Ok => {}
				x => return x,
			};
		}

		// Build graph and check for cycles.
		// This must be done last.
		let graph = self.build_graph();
		if let Err(cycle) = toposort(&graph, None) {
			return PipelineCheckResult::HasCycle {
				node: cycle.node_id().into(),
			};
		}

		self.check_state = PipelineCheckState::Passed;
		return PipelineCheckResult::Ok;
	}

	/// Given the global port state `port_data` and the node input mapping `input_map`,
	/// return the data that `input_port` consumes.
	///
	/// Returns `None` if this data is unavailable for any reason.
	fn get_node_input(
		port_data: &HashMap<PipelineNode, HashMap<PipelinePortLabel, Option<PipelineData>>>,
		input_map: &HashMap<PipelinePortLabel, NodeOutput>,
		input_port: &PipelinePortLabel,
	) -> Option<PipelineData> {
		match input_map.get(input_port) {
			None => None,
			Some(NodeOutput::InlineText { text }) => Some(PipelineData::Text(text.clone())),

			Some(NodeOutput::Node {
				node: PipelineNode::External,
				port,
			}) => port_data
				.get(&PipelineNode::External)
				.unwrap()
				.get(port)
				.cloned()
				.unwrap(),

			Some(NodeOutput::Node { node, port }) => {
				if let Some(x) = port_data.get(node) {
					if let Some(y) = x.get(port) {
						y.clone()
					} else {
						None
					}
				} else {
					None
				}
			}
		}
	}

	pub fn run(
		&self,
		inputs: HashMap<PipelinePortLabel, Option<PipelineData>>,
	) -> Result<HashMap<PipelinePortLabel, Option<PipelineData>>, PipelineError> {
		match self.check_state {
			PipelineCheckState::Failed => return Err(PipelineError::PipelineCheckFailed),
			PipelineCheckState::Unchecked => return Err(PipelineError::PipelineUnchecked),
			PipelineCheckState::Passed => {}
		};

		// TODO: parallelize
		let graph = self.build_graph();
		let node_order = toposort(&graph, None)
			.unwrap()
			.into_iter()
			.map(Into::<PipelineNode>::into);

		let mut port_data: HashMap<PipelineNode, _> = HashMap::new();
		port_data.insert(PipelineNode::External, inputs);

		for n in node_order {
			if n == PipelineNode::External {
				continue;
			}

			let node = self.nodes.get(n.to_label_ref().unwrap()).unwrap();
			let out = node.node_type.run(|label: &PipelinePortLabel| {
				Self::get_node_input(&port_data, &node.input, label)
			})?;

			port_data.insert(n, out);
		}

		let mut out = HashMap::new();
		for output_label in self.pipeline.outmap.keys() {
			out.insert(
				output_label.clone(),
				Self::get_node_input(&port_data, &self.pipeline.outmap, output_label),
			);
		}

		return Ok(out);
	}
}
