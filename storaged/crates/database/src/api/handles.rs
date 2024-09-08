#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// We don't derive ToSchema here, since utoipa doesn't
// take serde(transparent) into account.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClassId {
	id: u32,
}

impl From<ClassId> for u32 {
	fn from(value: ClassId) -> Self {
		value.id
	}
}

impl From<u32> for ClassId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId {
	id: u32,
}

impl From<ItemId> for u32 {
	fn from(value: ItemId) -> Self {
		value.id
	}
}

impl From<u32> for ItemId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}
