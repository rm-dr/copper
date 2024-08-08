//! Helpful types for node and port labels

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

/// A node label in a pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize, PartialOrd, Ord)]
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

impl From<String> for PipelineNodeLabel {
	fn from(s: String) -> Self {
		PipelineNodeLabel(s.into())
	}
}

impl<'a> From<&'a PipelineNodeLabel> for &'a str {
	fn from(value: &'a PipelineNodeLabel) -> Self {
		&value.0
	}
}

/// A port label in a pipeline
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize, PartialOrd, Ord)]
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

impl From<&SmartString<LazyCompact>> for PipelinePortLabel {
	fn from(s: &SmartString<LazyCompact>) -> Self {
		PipelinePortLabel(s.clone())
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

impl From<String> for PipelinePortLabel {
	fn from(s: String) -> Self {
		PipelinePortLabel(s.into())
	}
}

impl<'a> From<&'a PipelinePortLabel> for &'a str {
	fn from(value: &'a PipelinePortLabel) -> Self {
		&value.0
	}
}

/// A pipeline's name
#[derive(Debug, Hash, PartialEq, Eq, Clone, Deserialize, PartialOrd, Ord)]
pub struct PipelineLabel(SmartString<LazyCompact>);

impl Display for PipelineLabel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl AsRef<str> for PipelineLabel {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<SmartString<LazyCompact>> for PipelineLabel {
	fn from(s: SmartString<LazyCompact>) -> Self {
		PipelineLabel(s)
	}
}

impl From<PipelineLabel> for SmartString<LazyCompact> {
	fn from(value: PipelineLabel) -> Self {
		value.0
	}
}

impl From<&PipelineLabel> for SmartString<LazyCompact> {
	fn from(value: &PipelineLabel) -> Self {
		value.0.clone()
	}
}

impl From<&str> for PipelineLabel {
	fn from(s: &str) -> Self {
		PipelineLabel(s.into())
	}
}

impl From<String> for PipelineLabel {
	fn from(s: String) -> Self {
		PipelineLabel(s.into())
	}
}

impl<'a> From<&'a PipelineLabel> for &'a str {
	fn from(value: &'a PipelineLabel) -> Self {
		&value.0
	}
}
