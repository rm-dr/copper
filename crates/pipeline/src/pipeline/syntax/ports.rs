//! Helper structs for node inputs and outputs

use serde::{de::DeserializeOwned, Deserialize};
use std::{fmt::Debug, str::FromStr};

use crate::{
	api::PipelineNodeStub,
	labels::{PipelineNodeLabel, PipelinePortLabel},
};

/// An output port in the pipeline.
/// (i.e, a port that produces data.)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[serde(bound = "NodeStubType: DeserializeOwned")]
pub(crate) enum NodeOutput<NodeStubType: PipelineNodeStub> {
	/// An output port of the pipeline
	Pipeline {
		/// The port's name
		#[serde(rename = "pipeline")]
		port: PipelinePortLabel,
	},

	/// An output port of a node
	Node {
		/// The node that provides this output
		node: PipelineNodeLabel,

		/// The output's name
		#[serde(rename = "output")]
		port: PipelinePortLabel,
	},

	/// An inline node.
	/// This node must have EXACTLY one output.
	Inline(NodeStubType),
}

// TODO: better error
impl<NodeStubType: PipelineNodeStub> FromStr for NodeOutput<NodeStubType> {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split('.');
		let a = i.next();
		let b = i.next();

		if a.is_none() || b.is_none() || i.next().is_some() {
			return Err("bad link format".into());
		}
		let a = a.unwrap();
		let b = b.unwrap();

		Ok(Self::Node {
			node: a.into(),
			port: b.into(),
		})
	}
}

/// An input port in the pipeline.
/// (i.e, a port that consumes data.)
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum NodeInput {
	/// An output port of the pipeline
	Pipeline {
		#[serde(rename = "pipeline")]
		/// The port's name
		port: PipelinePortLabel,
	},

	/// An input port of a node
	Node {
		/// The node that provides this input
		node: PipelineNodeLabel,

		/// The port's name
		#[serde(rename = "input")]
		port: PipelinePortLabel,
	},
}

// TODO: better error
impl FromStr for NodeInput {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut i = s.split('.');
		let a = i.next();
		let b = i.next();

		if a.is_none() || b.is_none() || i.next().is_some() {
			return Err("bad link format".into());
		}
		let a = a.unwrap();
		let b = b.unwrap();

		Ok(Self::Node {
			node: a.into(),
			port: b.into(),
		})
	}
}
