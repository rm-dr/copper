//! Helpful types

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::fmt::Display;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeId(SmartString<LazyCompact>);

impl NodeId {
	/// Make a new pipeline node id
	pub fn new(id: &str) -> Self {
		Self(id.into())
	}

	/// get the id
	pub fn id(&self) -> &SmartString<LazyCompact> {
		&self.0
	}
}

impl Display for NodeId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl From<String> for NodeId {
	fn from(value: String) -> Self {
		Self::new(&value)
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PortName(SmartString<LazyCompact>);

impl PortName {
	/// Make a new pipeline port id
	pub fn new(id: &str) -> Self {
		Self(id.into())
	}

	/// get the id
	pub fn id(&self) -> &SmartString<LazyCompact> {
		&self.0
	}
}

impl Display for PortName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl From<String> for PortName {
	fn from(value: String) -> Self {
		Self::new(&value)
	}
}
