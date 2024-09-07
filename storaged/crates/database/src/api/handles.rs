#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// We don't derive ToSchema here, since utoipa doesn't
// take serde(transparent) into account.

/// The unique index of an item in it's class.
/// This does NOT identify an item uniquely; it identifies an item uniquely *in its class*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemIdx {
	id: u32,
}

impl From<ItemIdx> for u32 {
	fn from(value: ItemIdx) -> Self {
		value.id
	}
}

impl From<u32> for ItemIdx {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DatasetId {
	id: u32,
}

impl From<DatasetId> for u32 {
	fn from(value: DatasetId) -> Self {
		value.id
	}
}

impl From<u32> for DatasetId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemclassId {
	id: u32,
}

impl From<ItemclassId> for u32 {
	fn from(value: ItemclassId) -> Self {
		value.id
	}
}

impl From<u32> for ItemclassId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttributeId {
	id: u32,
}

impl From<AttributeId> for u32 {
	fn from(value: AttributeId) -> Self {
		value.id
	}
}

impl From<u32> for AttributeId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}
