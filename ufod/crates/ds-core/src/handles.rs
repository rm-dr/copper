use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The unique index of an item in it's class.
/// This does NOT identify an item uniquely; it identifies an item uniquely *in its class*.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
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
pub struct ClassHandle {
	id: u32,
}

impl From<ClassHandle> for u32 {
	fn from(value: ClassHandle) -> Self {
		value.id
	}
}

impl From<u32> for ClassHandle {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttrHandle {
	id: u32,
}

impl From<AttrHandle> for u32 {
	fn from(value: AttrHandle) -> Self {
		value.id
	}
}

impl From<u32> for AttrHandle {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}
