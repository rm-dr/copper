use std::collections::HashMap;

use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::Deserialize;

use super::{
	PipelineCheckResult, PipelineInput, PipelineNodeLabel, PipelineOutput, PipelinePortLabel,
};
use crate::pipeline::{
	data::{PipelineData, PipelineDataType},
	errors::PipelineError,
	nodes::PipelineNodes,
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
	pub outmap: HashMap<PipelinePortLabel, PipelineOutput>,
}

/// A pipeline node specification
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	pub node_type: PipelineNodes,

	/// Where this node should read its input from.
	#[serde(default)]
	pub input: HashMap<PipelinePortLabel, PipelineOutput>,
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

// TODO: rename: pipeline inputs are outputs
// TODO: pretty errors
// TODO: warnings (disconnected input)
// TODO: check for unused nodes
impl Pipeline {
	/// Check a link from `output` to `input`.
	/// Returns [`None`] if everything is ok, and an error otherwise.
	/// [`PipelineCheckResult::Ok`] will never be returned.
	///
	/// This makes sure that...
	/// - The output node exists
	/// - The output node has the specified port
	/// - The input node exists
	/// - The input node has the specified port
	/// - The input and output ports have matching types
	fn check_link(
		&self,
		output: &PipelineOutput,
		input: PipelineInput,
	) -> Option<PipelineCheckResult> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type = match output {
			PipelineOutput::InlineText { .. } => PipelineDataType::Text,
			PipelineOutput::Node { node, port } => {
				let get_node = self.nodes.get(node);

				if get_node.is_none() {
					return Some(PipelineCheckResult::NoNode {
						node: node.clone(),
						caused_by_input: input,
					});
				}
				let get_node = get_node.unwrap();
				let input_spec = get_node.node_type.get_output(port);

				if input_spec.is_none() {
					return Some(PipelineCheckResult::NoNodeOutput {
						node: node.clone(),
						output_name: port.clone(),
						caused_by_input: input,
					});
				}
				input_spec.unwrap()
			}
			PipelineOutput::Pinput { port } => {
				if let Some(from_type) = self.pipeline.input.get(port) {
					*from_type
				} else {
					return Some(PipelineCheckResult::NoPipelineInput {
						pipeline_input_name: port.clone(),
						caused_by_input: input,
					});
				}
			}
		};

		// Find the datatype of the input port we're connecting to.
		// While doing this, make sure both the input node and port exist.
		let input_type = match &input {
			PipelineInput::Node { node, port } => {
				let get_node = self.nodes.get(node);

				if get_node.is_none() {
					return Some(PipelineCheckResult::NoNode {
						node: node.clone(),
						caused_by_input: input,
					});
				}
				let get_node = get_node.unwrap();
				let input = get_node.node_type.get_input(port);

				if input.is_none() {
					return Some(PipelineCheckResult::NoNodeInput {
						node: node.clone(),
						input_name: port.clone(),
					});
				}
				input.unwrap()
			}
			PipelineInput::Poutput { port } => {
				if let Some(from_type) = self.pipeline.output.get(port) {
					*from_type
				} else {
					return Some(PipelineCheckResult::NoPipelineOutput {
						pipeline_output_name: port.clone(),
					});
				}
			}
		};

		// Check types
		match output {
			PipelineOutput::InlineText { .. } => {
				if input_type != PipelineDataType::Text {
					return Some(PipelineCheckResult::InlineTypeMismatch {
						inline_type: PipelineDataType::Text,
						input,
					});
				}
			}
			PipelineOutput::Pinput { .. } | PipelineOutput::Node { .. } => {
				if output_type != input_type {
					return Some(PipelineCheckResult::TypeMismatch {
						output: output.clone(),
						input,
					});
				}
			}
		};

		return None;
	}

	/// Build
	fn build_graph(&self) -> GraphMap<&str, (), Directed> {
		// We don't need to create nodes explicitly,
		// since `add_edge` does this automatically.
		let mut graph = GraphMap::<&str, (), Directed>::new();
		for (node_name, node_spec) in &self.nodes {
			for out_link in node_spec.input.values() {
				match out_link {
					PipelineOutput::InlineText { .. } => {}
					PipelineOutput::Node { .. } | PipelineOutput::Pinput { .. } => {
						graph.add_edge(out_link.node_str().unwrap(), node_name.into(), ());
					}
				}
			}
		}

		return graph;
	}

	pub fn check(&mut self) -> PipelineCheckResult {
		self.check_state = PipelineCheckState::Failed;

		// Check each node's inputs
		for (node_name, node_spec) in &self.nodes {
			for (input_name, out_link) in &node_spec.input {
				if let Some(err) = self.check_link(
					out_link,
					PipelineInput::Node {
						node: node_name.clone(),
						port: input_name.clone(),
					},
				) {
					return err;
				};
			}
		}

		// Check final pipeline outputs
		for (out_name, out_link) in &self.pipeline.outmap {
			if let Some(err) = self.check_link(
				out_link,
				PipelineInput::Poutput {
					port: out_name.clone(),
				},
			) {
				return err;
			};
		}

		// Build graph and check for cycles
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
		port_data: &HashMap<PipelineNodeLabel, HashMap<PipelinePortLabel, Option<PipelineData>>>,
		input_map: &HashMap<PipelinePortLabel, PipelineOutput>,
		input_port: &PipelinePortLabel,
	) -> Option<PipelineData> {
		match input_map.get(input_port) {
			None => None,
			Some(PipelineOutput::InlineText { text }) => Some(PipelineData::Text(text.clone())),
			Some(PipelineOutput::Node { node, port }) => {
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
			Some(PipelineOutput::Pinput { port }) => port_data
				.get(&"in".into())
				.unwrap()
				.get(port)
				.cloned()
				.unwrap(),
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
			.map(Into::<PipelineNodeLabel>::into);

		let mut port_data: HashMap<PipelineNodeLabel, _> = HashMap::new();
		port_data.insert("in".into(), inputs);

		for n in node_order {
			if n == "in".into() || n == "out".into() {
				continue;
			}

			let node = self.nodes.get(&n).unwrap();
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
