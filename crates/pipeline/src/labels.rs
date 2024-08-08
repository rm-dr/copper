//! Helpful types

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

/// A pipeline node's id
#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PipelineNodeID(SmartString<LazyCompact>);

impl PipelineNodeID {
	/// Make a new pipeline node id
	pub fn new(id: &str) -> Self {
		Self(id.into())
	}

	/// get the id
	pub fn id(&self) -> &SmartString<LazyCompact> {
		&self.0
	}
}

impl Display for PipelineNodeID {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl From<String> for PipelineNodeID {
	fn from(value: String) -> Self {
		Self::new(&value)
	}
}

/// A pipeline node's port's id
#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PipelinePortID(SmartString<LazyCompact>);

impl PipelinePortID {
	/// Make a new pipeline port id
	pub fn new(id: &str) -> Self {
		Self(id.into())
	}

	/// get the id
	pub fn id(&self) -> &SmartString<LazyCompact> {
		&self.0
	}
}

impl Display for PipelinePortID {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl From<String> for PipelinePortID {
	fn from(value: String) -> Self {
		Self::new(&value)
	}
}

/// A pipeline's name
#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PipelineName(SmartString<LazyCompact>);

impl PipelineName {
	/// Make a new pipeline name
	pub fn new(name: &str) -> Self {
		Self(name.into())
	}

	/// Get the name
	pub fn name(&self) -> &SmartString<LazyCompact> {
		&self.0
	}
}

impl Display for PipelineName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl From<String> for PipelineName {
	fn from(value: String) -> Self {
		Self::new(&value)
	}
}
