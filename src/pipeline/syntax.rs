use petgraph::{algo::toposort, graphmap::GraphMap, Directed};
use serde::{
	de::{self},
	Deserialize, Deserializer,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::HashMap, fmt::Display};

use super::{nodes::PipelineNodes, PipelineDataType, PortLink};

#[derive(Debug, Deserialize)]
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
		caused_by_input: PortLink,
	},

	/// `node` has no input named `input_name`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: PipelineNodeSpec,
		input_name: SmartString<LazyCompact>,
	},

	/// `node` has no output named `output_name`.
	/// We tried to connect this output from `caused_by_input`.
	NoNodeOutput {
		node: PipelineNodeSpec,
		output_name: SmartString<LazyCompact>,
		caused_by_input: PortLink,
	},

	/// This pipeline has no input named `input_name`.
	/// We tried to connect to this input from `caused_by_input`.
	NoPipelineInput {
		pipeline_input_name: SmartString<LazyCompact>,
		caused_by_input: PortLink,
	},

	/// This pipeline has no output named `output_name`.
	NoPipelineOutput {
		pipeline_output_name: SmartString<LazyCompact>,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch { output: PortLink, input: PortLink },

	/// We tried to connect an inline type to `input`,
	/// but their types don't match.
	InlineTypeMismatch {
		inline_type: PipelineDataType,
		input: PortLink,
	},

	/// This graph has a cycle containing `node`
	HasCycle { node: PipelineNodeSpec },
}

// TODO: rename: pipeline inputs are outputs
// TODO: pretty errors
// TODO: warnings (disconnected input)
// TODO: rework `PipelineLink`
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
	fn check_link(&self, output: &PipelineLink, input: PortLink) -> Option<PipelineCheckResult> {
		// Find the datatype of the output port we're connecting to.
		// While doing this, make sure both the output node and port exist.
		let output_type = match output {
			PipelineLink::InlineText { .. } => PipelineDataType::Text,
			PipelineLink::Link(link) => match link {
				PortLink::Node { node, port } => {
					let get_node = self.nodes.get(node);

					if get_node.is_none() {
						return Some(PipelineCheckResult::NoNode {
							node: node.clone(),
							caused_by_input: input,
						});
					}
					let node = get_node.unwrap();
					let input_spec = node.node_type.get_outputs().iter().find(|x| x.0 == port);

					if input_spec.is_none() {
						return Some(PipelineCheckResult::NoNodeOutput {
							node: node.clone(),
							output_name: port.clone(),
							caused_by_input: input,
						});
					}
					input_spec.unwrap().1
				}
				PortLink::Pinput { port } => {
					if let Some(from_type) = self.pipeline.input.get(port) {
						*from_type
					} else {
						return Some(PipelineCheckResult::NoPipelineInput {
							pipeline_input_name: port.clone(),
							caused_by_input: input,
						});
					}
				}
				PortLink::Poutput { .. } => unreachable!("an output was connected to an output!"),
			},
		};

		// Find the datatype of the input port we're connecting to.
		// While doing this, make sure both the input node and port exist.
		let input_type = match &input {
			PortLink::Node { node, port } => {
				let get_node = self.nodes.get(node);

				if get_node.is_none() {
					return Some(PipelineCheckResult::NoNode {
						node: node.clone(),
						caused_by_input: input,
					});
				}
				let node = get_node.unwrap();
				let input = node.node_type.get_inputs().iter().find(|x| x.0 == port);

				if input.is_none() {
					return Some(PipelineCheckResult::NoNodeInput {
						node: node.clone(),
						input_name: port.clone(),
					});
				}
				input.unwrap().1
			}
			PortLink::Poutput { port } => {
				if let Some(from_type) = self.pipeline.output.get(port) {
					*from_type
				} else {
					return Some(PipelineCheckResult::NoPipelineOutput {
						pipeline_output_name: port.clone(),
					});
				}
			}
			PortLink::Pinput { .. } => unreachable!("an input was connected to an input!"),
		};

		// Check types
		match output {
			PipelineLink::InlineText { .. } => {
				if input_type != PipelineDataType::Text {
					return Some(PipelineCheckResult::InlineTypeMismatch {
						inline_type: PipelineDataType::Text,
						input,
					});
				}
			}
			PipelineLink::Link(link) => {
				if output_type != input_type {
					return Some(PipelineCheckResult::TypeMismatch {
						output: link.clone(),
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
					PortLink::Node {
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
				PortLink::Poutput {
					port: out_name.clone(),
				},
			) {
				return err;
			};
		}

		// Build graph...
		let mut deps = GraphMap::<&str, (), Directed>::new();
		self.nodes.iter().for_each(|(node_name, _)| {
			deps.add_node(node_name);
		});

		for (node_name, node_spec) in &self.nodes {
			for (_input_name, out_link) in &node_spec.input {
				match out_link {
					PipelineLink::Link(link) => {
						deps.add_edge(link.node_str(), node_name, ());
					}
					_ => {}
				}
			}
		}

		// ...and check for cycles.
		let topo = toposort(&deps, None);
		if let Err(cycle) = topo {
			return PipelineCheckResult::HasCycle {
				node: self.nodes.get(cycle.node_id()).unwrap().clone(),
			};
		}

		return PipelineCheckResult::Ok {
			topo: topo.unwrap().into_iter().map(|x| x.into()).collect(),
		};
	}
}

#[derive(Debug, Deserialize)]
pub struct PipelineConfig {
	/// Names and types of pipeline inputs
	#[serde(default)]
	pub input: HashMap<SmartString<LazyCompact>, PipelineDataType>,

	/// Names and types of pipeline outputs
	#[serde(default)]
	pub output: HashMap<SmartString<LazyCompact>, PipelineDataType>,

	/// Map pipeline outputs to the node outputs that produce them
	#[serde(default)]
	pub outmap: HashMap<SmartString<LazyCompact>, PipelineLink>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	pub node_type: PipelineNodes,

	/// Where this node should read its input from.
	#[serde(default)]
	pub input: HashMap<SmartString<LazyCompact>, PipelineLink>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum PipelineLink {
	/// Inline static text
	InlineText { text: String },

	/// Get data from another node's output
	#[serde(deserialize_with = "parse_link")]
	Link(PortLink),
}

impl Display for PipelineLink {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InlineText { text } => write!(f, "InlineText(\"{text}\")"),
			Self::Link(link) => write!(f, "Portlink({link})"),
		}
	}
}

// TODO: handle "in" links
// TODO: type for "in" links?
fn parse_link<'de, D>(deserializer: D) -> Result<PortLink, D::Error>
where
	D: Deserializer<'de>,
{
	let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
	let mut i = addr_str.split('.');
	let a = i.next();
	let b = i.next();

	if a.is_none() || b.is_none() || i.next().is_some() {
		return Err(de::Error::custom("bad link format"));
	}
	let a = a.unwrap();
	let b = b.unwrap();

	Ok(match a {
		"in" => PortLink::Pinput { port: b.into() },
		"out" => PortLink::Poutput { port: b.into() },
		_ => PortLink::Node {
			node: a.into(),
			port: b.into(),
		},
	})
}
