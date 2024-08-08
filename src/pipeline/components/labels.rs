use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

/// Reserved name for a pipeline's input node
pub const PIPELINE_NODE_NAME: &str = "pipeline";

/// A node label in a pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PipelineNode {
	/// This pipeline's external interface.
	///
	/// This node's outputs are the data provided to the pipeline,
	/// and its inputs are the data this pipeline produces.
	OuterNode,

	/// A named node in this pipeline
	Node(SmartString<LazyCompact>),
}

impl<'de> Deserialize<'de> for PipelineNode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		return Ok(addr_str.into());
	}
}

impl Display for PipelineNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Node(x) => x.fmt(f),
			Self::OuterNode => write!(f, "{}", PIPELINE_NODE_NAME),
		}
	}
}

impl AsRef<str> for PipelineNode {
	fn as_ref(&self) -> &str {
		match self {
			Self::Node(x) => x,
			Self::OuterNode => PIPELINE_NODE_NAME,
		}
	}
}

impl From<&str> for PipelineNode {
	fn from(s: &str) -> Self {
		if s == PIPELINE_NODE_NAME {
			PipelineNode::OuterNode
		} else {
			PipelineNode::Node(s.into())
		}
	}
}

impl From<SmartString<LazyCompact>> for PipelineNode {
	fn from(s: SmartString<LazyCompact>) -> Self {
		Self::from(&s[..])
	}
}

impl From<PipelineNode> for SmartString<LazyCompact> {
	fn from(value: PipelineNode) -> Self {
		match value {
			PipelineNode::Node(x) => x,
			PipelineNode::OuterNode => PIPELINE_NODE_NAME.into(),
		}
	}
}

impl From<&PipelineNode> for SmartString<LazyCompact> {
	fn from(value: &PipelineNode) -> Self {
		match value {
			PipelineNode::Node(x) => x.clone(),
			PipelineNode::OuterNode => PIPELINE_NODE_NAME.into(),
		}
	}
}

impl<'a> From<&'a PipelineNode> for &'a str {
	fn from(value: &'a PipelineNode) -> Self {
		match value {
			PipelineNode::Node(x) => x,
			PipelineNode::OuterNode => PIPELINE_NODE_NAME,
		}
	}
}

/// A port label in a pipeline pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize)]
pub struct PipelinePort(SmartString<LazyCompact>);

impl Display for PipelinePort {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl AsRef<str> for PipelinePort {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<SmartString<LazyCompact>> for PipelinePort {
	fn from(s: SmartString<LazyCompact>) -> Self {
		PipelinePort(s)
	}
}

impl From<PipelinePort> for SmartString<LazyCompact> {
	fn from(value: PipelinePort) -> Self {
		value.0
	}
}

impl From<&PipelinePort> for SmartString<LazyCompact> {
	fn from(value: &PipelinePort) -> Self {
		value.0.clone()
	}
}

impl From<&str> for PipelinePort {
	fn from(s: &str) -> Self {
		PipelinePort(s.into())
	}
}

impl<'a> From<&'a PipelinePort> for &'a str {
	fn from(value: &'a PipelinePort) -> Self {
		&value.0
	}
}
