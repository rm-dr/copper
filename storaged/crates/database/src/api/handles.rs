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
pub struct DatasetHandle {
	id: u32,
}

impl From<DatasetHandle> for u32 {
	fn from(value: DatasetHandle) -> Self {
		value.id
	}
}

impl From<u32> for DatasetHandle {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemclassHandle {
	id: u32,
}

impl From<ItemclassHandle> for u32 {
	fn from(value: ItemclassHandle) -> Self {
		value.id
	}
}

impl From<u32> for ItemclassHandle {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttributeHandle {
	id: u32,
}

impl From<AttributeHandle> for u32 {
	fn from(value: AttributeHandle) -> Self {
		value.id
	}
}

impl From<u32> for AttributeHandle {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}
