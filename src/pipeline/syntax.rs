use serde::{
	de::{self},
	Deserialize, Deserializer,
};
use std::{collections::HashMap, fmt::Display};

use super::{nodes::PipelineNodes, PipelineDataType, PortLink};

#[derive(Debug, Deserialize)]
pub struct Pipeline {
	/// Pipeline parameters
	pub pipeline: PipelineConfig,

	/// Nodes in this pipeline
	#[serde(default)]
	#[serde(rename = "node")]
	pub nodes: HashMap<String, PipelineNodeSpec>,
}

#[derive(Debug)]
pub enum PipelineCheckResult {
	Ok,

	/// There is no node named `node` in this pipeline
	/// We tried to connect this node from `caused_by_input`.
	NoNode {
		node: String,
		caused_by_input: PortLink,
	},

	/// `node` has no input named `input_name`.
	/// This is triggered when we specify an input that doesn't exist.
	NoNodeInput {
		node: PipelineNodeSpec,
		input_name: String,
	},

	/// `node` has no output named `output_name`lf
	/// We tried to connect this output from `caused_by_input`.
	NoNodeOutput {
		node: PipelineNodeSpec,
		output_name: String,
		caused_by_input: PortLink,
	},

	/// pipeline has no input named `input_name`.
	/// We tried to connect to this input from `caused_by_input`.
	NoPipelineInput {
		pipeline_input_name: String,
		caused_by_input: PortLink,
	},

	/// We tried to connect `input` to `output`,
	/// but their types don't match.
	TypeMismatch {
		output: PortLink,
		input: PortLink,
	},
}

// TODO: check for cycles
// TODO: rename: pipeline inputs are outputs
// TODO: pretty errors
// TODO: warnings (disconnected input)
impl Pipeline {
	pub fn check(&self) -> PipelineCheckResult {
		// Check each nodes's input
		for (node_name, node_spec) in &self.nodes {
			for (input_name, out_link) in &node_spec.input {
				// input_name: the name of THIS nodes's input we're connecting
				// out_link: the node `input_name` is connected to

				// Make sure `input_name` is a valid input for this node
				if node_spec
					.node_type
					.get_inputs()
					.iter()
					.find(|x| x.0 == &input_name[..])
					.is_none()
				{
					return PipelineCheckResult::NoNodeInput {
						node: node_spec.clone(),
						input_name: input_name.clone(),
					};
				}

				let in_type = &node_spec
					.node_type
					.get_inputs()
					.iter()
					.find(|x| x.0 == &input_name[..])
					.unwrap()
					.1;

				// Make sure `out_link` is valid
				match out_link {
					PipelineLink::InlineText { .. } => {}
					PipelineLink::Link(link) => {
						// Special case: we're linked to pipeline input
						if link.node == "in" {
							let input = self.pipeline.input.get(&link.port[..]);

							if let Some(out_type) = input {
								// Make sure input type matches output type
								if in_type != out_type {
									return PipelineCheckResult::TypeMismatch {
										output: link.clone(),
										input: PortLink {
											node: node_name.clone(),
											port: input_name.clone(),
										},
									};
								}
							} else {
								return PipelineCheckResult::NoPipelineInput {
									pipeline_input_name: link.port.clone(),
									caused_by_input: PortLink {
										node: node_name.clone(),
										port: input_name.clone(),
									},
								};
							}

						// We're linked to another node's output
						} else {
							let source_node = self.nodes.get(&link.node);

							if let Some(source_node) = source_node {
								let output = source_node
									.node_type
									.get_outputs()
									.iter()
									.find(|x| x.0 == &link.port[..]);

								if let Some(output) = output {
									let out_type = &output.1;
									if in_type != out_type {
										return PipelineCheckResult::TypeMismatch {
											output: link.clone(),
											input: PortLink {
												node: node_name.clone(),
												port: input_name.clone(),
											},
										};
									}
								} else {
									return PipelineCheckResult::NoNodeOutput {
										node: source_node.clone(),
										output_name: link.port.clone(),
										caused_by_input: PortLink {
											node: node_name.clone(),
											port: input_name.clone(),
										},
									};
								}
							} else {
								return PipelineCheckResult::NoNode {
									node: link.node.clone(),
									caused_by_input: PortLink {
										node: node_name.clone(),
										port: input_name.clone(),
									},
								};
							}
						}
					}
				};
			}
		}

		return PipelineCheckResult::Ok;
	}
}

#[derive(Debug, Deserialize)]
pub struct PipelineConfig {
	/// Names and types of pipeline inputs
	#[serde(default)]
	pub input: HashMap<String, PipelineDataType>,

	/// Names and types of pipeline outputs
	#[serde(default)]
	pub output: HashMap<String, PipelineDataType>,

	/// Map pipeline outputs to the node outputs that produce them
	#[serde(default)]
	pub outmap: HashMap<String, PipelineLink>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PipelineNodeSpec {
	/// What kind of node is this?
	#[serde(rename = "type")]
	pub node_type: PipelineNodes,

	/// Where this node should read its input from.
	#[serde(default)]
	pub input: HashMap<String, PipelineLink>,
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

fn parse_link<'de, D>(deserializer: D) -> Result<PortLink, D::Error>
where
	D: Deserializer<'de>,
{
	let addr_str = String::deserialize(deserializer)?;
	let mut i = addr_str.split('.');
	let a = i.next();
	let b = i.next();

	if a.is_none() || b.is_none() || i.next().is_some() {
		return Err("bad link format").map_err(de::Error::custom);
	}

	Ok(PortLink {
		node: a.unwrap().to_string(),
		port: b.unwrap().to_string(),
	})
}
