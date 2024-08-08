use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

/// Reserved name for a pipeline's input node
pub const PIPELINE_EXTERNAL_NODE_NAME: &str = "pipeline";

/// A node label in a pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PipelineNode {
	/// This pipeline's external interface.
	///
	/// This node's outputs are the data provided to the pipeline,
	/// and its inputs are the data this pipeline produces.
	External,

	/// A named node in this pipeline
	Node(PipelineNodeLabel),
}

impl PipelineNode {
	/// Convert this into a [`PipelineNodeLabel`].
	/// Returns `None` if this is a [`PipelineNode::External`].
	pub fn to_label(&self) -> Option<PipelineNodeLabel> {
		match self {
			Self::External => None,
			Self::Node(x) => Some(x.clone()),
		}
	}

	pub fn to_label_ref(&self) -> Option<&PipelineNodeLabel> {
		match self {
			Self::External => None,
			Self::Node(x) => Some(x),
		}
	}
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
			Self::External => write!(f, "{}", PIPELINE_EXTERNAL_NODE_NAME),
		}
	}
}

impl From<PipelineNodeLabel> for PipelineNode {
	fn from(value: PipelineNodeLabel) -> Self {
		Self::Node(value)
	}
}

impl From<&PipelineNodeLabel> for PipelineNode {
	fn from(value: &PipelineNodeLabel) -> Self {
		Self::Node(value.clone())
	}
}

impl AsRef<str> for PipelineNode {
	fn as_ref(&self) -> &str {
		match self {
			Self::Node(x) => x.into(),
			Self::External => PIPELINE_EXTERNAL_NODE_NAME,
		}
	}
}

impl From<&str> for PipelineNode {
	fn from(s: &str) -> Self {
		if s == PIPELINE_EXTERNAL_NODE_NAME {
			PipelineNode::External
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
			PipelineNode::Node(x) => x.into(),
			PipelineNode::External => PIPELINE_EXTERNAL_NODE_NAME.into(),
		}
	}
}

impl From<&PipelineNode> for SmartString<LazyCompact> {
	fn from(value: &PipelineNode) -> Self {
		match value {
			PipelineNode::Node(x) => x.into(),
			PipelineNode::External => PIPELINE_EXTERNAL_NODE_NAME.into(),
		}
	}
}

impl<'a> From<&'a PipelineNode> for &'a str {
	fn from(value: &'a PipelineNode) -> Self {
		match value {
			PipelineNode::Node(x) => x.into(),
			PipelineNode::External => PIPELINE_EXTERNAL_NODE_NAME,
		}
	}
}

/// A node label in a pipeline pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize)]
pub struct PipelineNodeLabel(SmartString<LazyCompact>);

impl Display for PipelineNodeLabel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl AsRef<str> for PipelineNodeLabel {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<SmartString<LazyCompact>> for PipelineNodeLabel {
	fn from(s: SmartString<LazyCompact>) -> Self {
		PipelineNodeLabel(s)
	}
}

impl From<PipelineNodeLabel> for SmartString<LazyCompact> {
	fn from(value: PipelineNodeLabel) -> Self {
		value.0
	}
}

impl From<&PipelineNodeLabel> for SmartString<LazyCompact> {
	fn from(value: &PipelineNodeLabel) -> Self {
		value.0.clone()
	}
}

impl From<&str> for PipelineNodeLabel {
	fn from(s: &str) -> Self {
		PipelineNodeLabel(s.into())
	}
}

impl<'a> From<&'a PipelineNodeLabel> for &'a str {
	fn from(value: &'a PipelineNodeLabel) -> Self {
		&value.0
	}
}

/// A port label in a pipeline pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize)]
pub struct PipelinePortLabel(SmartString<LazyCompact>);

impl Display for PipelinePortLabel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl AsRef<str> for PipelinePortLabel {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<SmartString<LazyCompact>> for PipelinePortLabel {
	fn from(s: SmartString<LazyCompact>) -> Self {
		PipelinePortLabel(s)
	}
}

impl From<PipelinePortLabel> for SmartString<LazyCompact> {
	fn from(value: PipelinePortLabel) -> Self {
		value.0
	}
}

impl From<&PipelinePortLabel> for SmartString<LazyCompact> {
	fn from(value: &PipelinePortLabel) -> Self {
		value.0.clone()
	}
}

impl From<&str> for PipelinePortLabel {
	fn from(s: &str) -> Self {
		PipelinePortLabel(s.into())
	}
}

impl<'a> From<&'a PipelinePortLabel> for &'a str {
	fn from(value: &'a PipelinePortLabel) -> Self {
		&value.0
	}
}
