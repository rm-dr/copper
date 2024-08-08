use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

/// Reserved name for a pipeline's input node
pub const PIPELINE_NODE_NAME: &str = "pipeline";

/// A node label in a pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum PipelineNodeLabel {
	/// This pipeline's external interface.
	///
	/// This node's outputs are the data provided to the pipeline,
	/// and its inputs are the data this pipeline produces.
	PipelineNode,

	/// A named node in this pipeline
	Node(SmartString<LazyCompact>),
}

impl<'de> Deserialize<'de> for PipelineNodeLabel {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let addr_str = SmartString::<LazyCompact>::deserialize(deserializer)?;
		return Ok(addr_str.into());
	}
}

impl Display for PipelineNodeLabel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Node(x) => x.fmt(f),
			Self::PipelineNode => write!(f, "{}", PIPELINE_NODE_NAME),
		}
	}
}

impl AsRef<str> for PipelineNodeLabel {
	fn as_ref(&self) -> &str {
		match self {
			Self::Node(x) => x,
			Self::PipelineNode => PIPELINE_NODE_NAME,
		}
	}
}

impl From<&str> for PipelineNodeLabel {
	fn from(s: &str) -> Self {
		if s == PIPELINE_NODE_NAME {
			PipelineNodeLabel::PipelineNode
		} else {
			PipelineNodeLabel::Node(s.into())
		}
	}
}

impl From<SmartString<LazyCompact>> for PipelineNodeLabel {
	fn from(s: SmartString<LazyCompact>) -> Self {
		Self::from(&s[..])
	}
}

impl From<PipelineNodeLabel> for SmartString<LazyCompact> {
	fn from(value: PipelineNodeLabel) -> Self {
		match value {
			PipelineNodeLabel::Node(x) => x,
			PipelineNodeLabel::PipelineNode => PIPELINE_NODE_NAME.into(),
		}
	}
}

impl From<&PipelineNodeLabel> for SmartString<LazyCompact> {
	fn from(value: &PipelineNodeLabel) -> Self {
		match value {
			PipelineNodeLabel::Node(x) => x.clone(),
			PipelineNodeLabel::PipelineNode => PIPELINE_NODE_NAME.into(),
		}
	}
}

impl<'a> From<&'a PipelineNodeLabel> for &'a str {
	fn from(value: &'a PipelineNodeLabel) -> Self {
		match value {
			PipelineNodeLabel::Node(x) => x,
			PipelineNodeLabel::PipelineNode => PIPELINE_NODE_NAME,
		}
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
