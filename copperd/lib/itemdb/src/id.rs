#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// We don't derive ToSchema here, since utoipa doesn't
// take serde(transparent) into account.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DatasetId {
	id: i64,
}

impl From<DatasetId> for i64 {
	fn from(value: DatasetId) -> Self {
		value.id
	}
}

impl From<i64> for DatasetId {
	fn from(value: i64) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClassId {
	id: i64,
}

impl From<ClassId> for i64 {
	fn from(value: ClassId) -> Self {
		value.id
	}
}

impl From<i64> for ClassId {
	fn from(value: i64) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttributeId {
	id: i64,
}

impl From<AttributeId> for i64 {
	fn from(value: AttributeId) -> Self {
		value.id
	}
}

impl From<i64> for AttributeId {
	fn from(value: i64) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemId {
	id: i64,
}

impl From<ItemId> for i64 {
	fn from(value: ItemId) -> Self {
		value.id
	}
}

impl From<i64> for ItemId {
	fn from(value: i64) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId {
	id: i64,
}

impl From<UserId> for i64 {
	fn from(value: UserId) -> Self {
		value.id
	}
}

impl From<i64> for UserId {
	fn from(value: i64) -> Self {
		Self { id: value }
	}
}
