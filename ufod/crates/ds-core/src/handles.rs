use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemHandle {
	id: u32,
}

impl From<ItemHandle> for u32 {
	fn from(value: ItemHandle) -> Self {
		value.id
	}
}

impl From<u32> for ItemHandle {
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
