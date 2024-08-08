use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::HashMap;

use super::{nodes::PipelineNodes, PipelineDataType, PipelineInput, PipelineOutput};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pipeline {
	/// Pipeline parameters
	pub pipeline: PipelineConfig,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	pub nodes: HashMap<SmartString<LazyCompact>, PipelineNodeSpec>,
}

#[derive(Debug)]
pub enum PipelineCheckResult {
	/// This pipeline is good to go.
	Ok {
		/// A vector of all nodes in this pipeline in topological order:
		/// each node is ordered before its successors.
		topo: Vec<SmartString<LazyCompact>>,
	},

	/// There is no node named `node` in this pipeline
	/// We tried to connect this node from `caused_by_input`.
	NoNode {
		node: SmartString<LazyCompact>,
		caused_by_input: PipelineInput,
	},

	/// `node` has no input named `input_name`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: SmartString<LazyCompact>,
		input_name: SmartString<LazyCompact>,
	},

	/// `node` has no output named `output_name`.
	/// We tried to connect this output from `caused_by_input`.
	NoNodeOutput {
		node: SmartString<LazyCompact>,
		output_name: SmartString<LazyCompact>,
		caused_by_input: PipelineInput,
	},

	/// This pipeline has no input named `input_name`.
	/// We tried to connect to this input from `caused_by_input`.
	NoPipelineInput {
		pipeline_input_name: SmartString<LazyCompact>,
		caused_by_input: PipelineInput,
	},

	/// This pipeline has no output named `output_name`.
	NoPipelineOutput {
		pipeline_output_name: SmartString<LazyCompact>,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		output: PipelineOutput,
		input: PipelineInput,
	},

	/// We tried to connect an inline type to `input`,
	/// but their types don't match.
	InlineTypeMismatch {
		inline_type: PipelineDataType,
		input: PipelineInput,
	},

	/// This graph has a cycle containing `node`
	HasCycle { node: SmartString<LazyCompact> },
}

// TODO: rename: pipeline inputs are outputs
// TODO: pretty errors
// TODO: warnings (disconnected input)
// TODO: check for unused nodes
// TODO: add name to nodespec
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
				let input_spec = get_node
					.node_type
					.get_outputs()
					.iter()
					.find(|x| x.0 == port);

				if input_spec.is_none() {
					return Some(PipelineCheckResult::NoNodeOutput {
						node: node.clone(),
						output_name: port.clone(),
						caused_by_input: input,
					});
				}
				input_spec.unwrap().1
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
				let input = get_node.node_type.get_inputs().iter().find(|x| x.0 == port);

				if input.is_none() {
					return Some(PipelineCheckResult::NoNodeInput {
						node: node.clone(),
						input_name: port.clone(),
					});
				}
				input.unwrap().1
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

	pub fn check(&self) -> PipelineCheckResult {
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

		// Build graph...
		// We don't need to create nodes explicitly,
		// since `add_edge` does this automatically.
		let mut deps = GraphMap::<&str, (), Directed>::new();
		for (node_name, node_spec) in &self.nodes {
			for out_link in node_spec.input.values() {
				match out_link {
					PipelineOutput::InlineText { .. } => {}
					PipelineOutput::Node { .. } | PipelineOutput::Pinput { .. } => {
						deps.add_edge(out_link.node_str().unwrap(), node_name, ());
					}
				}
			}
		}

		// ...and check for cycles.
		let topo = toposort(&deps, None);
		if let Err(cycle) = topo {
			return PipelineCheckResult::HasCycle {
				node: cycle.node_id().into(),
			};
		}

		return PipelineCheckResult::Ok {
			topo: topo.unwrap().into_iter().map(|x| x.into()).collect(),
		};
	}
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
	/// Names and types of pipeline inputs
	#[serde(default)]
	pub input: HashMap<SmartString<LazyCompact>, PipelineDataType>,

	/// Names and types of pipeline outputs
	#[serde(default)]
	pub output: HashMap<SmartString<LazyCompact>, PipelineDataType>,

	/// Map pipeline outputs to the node outputs that produce them
	#[serde(default)]
	pub outmap: HashMap<SmartString<LazyCompact>, PipelineOutput>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	pub node_type: PipelineNodes,

	/// Where this node should read its input from.
	#[serde(default)]
	pub input: HashMap<SmartString<LazyCompact>, PipelineOutput>,
}
