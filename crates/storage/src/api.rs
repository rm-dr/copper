use std::{fmt::Debug, hash::Hash};

use crate::{
	data::{StorageData, StorageDataStub},
	errors::DatasetError,
};

pub struct AttributeOptions {
	pub(crate) unique: bool,
	pub(crate) not_null: bool,
}

impl Default for AttributeOptions {
	fn default() -> Self {
		Self {
			unique: false,
			not_null: false,
		}
	}
}

impl AttributeOptions {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn unique(mut self, is_unique: bool) -> Self {
		self.unique = is_unique;
		self
	}

	pub fn not_null(mut self, not_null: bool) -> Self {
		self.not_null = not_null;
		self
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

// TODO: better db backend. EAV is slow.
pub trait Dataset {
	fn add_class(&mut self, name: &str) -> Result<ClassHandle, DatasetError>;
	fn add_item(
		&mut self,
		class: ClassHandle,
		attrs: &[(AttrHandle, StorageData)],
	) -> Result<ItemHandle, DatasetError>;
	fn add_attr(
		&mut self,
		class: ClassHandle,
		name: &str,
		data_type: StorageDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, DatasetError>;

	fn del_class(&mut self, class: ClassHandle) -> Result<(), DatasetError>;
	fn del_item(&mut self, item: ItemHandle) -> Result<(), DatasetError>;
	fn del_attr(&mut self, attr: AttrHandle) -> Result<(), DatasetError>;

	//fn iter_items(&self) -> Result<impl Iterator<Item = ItemHandle>, ()>;
	//fn iter_classes(&self) -> Result<impl Iterator<Item = ClassHandle>, ()>;
	//fn iter_attrs(&self) -> Result<impl Iterator<Item = AttrHandle>, ()>;

	fn get_class(&mut self, class_name: &str) -> Result<Option<ClassHandle>, DatasetError>;
	fn get_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrHandle>, DatasetError>;

	// TODO: take &[(_, _)] instead of data
	fn item_set_attr(&mut self, attr: AttrHandle, data: &StorageData) -> Result<(), DatasetError>;
	fn item_get_attr(
		&self,
		item: ItemHandle,
		attr: AttrHandle,
	) -> Result<StorageData, DatasetError>;
	fn item_get_class(&self, item: ItemHandle) -> Result<ClassHandle, DatasetError>;

	fn class_set_name(&mut self, class: ClassHandle, name: &str) -> Result<(), DatasetError>;
	fn class_get_name(&self, class: ClassHandle) -> Result<&str, DatasetError>;
	//fn class_get_attrs(&self, class: ClassHandle) -> Result<impl Iterator<Item = AttrHandle>, ()>;
	fn class_num_attrs(&self, class: ClassHandle) -> Result<usize, DatasetError>;

	fn attr_set_name(&mut self, attr: AttrHandle, name: &str) -> Result<(), DatasetError>;
	fn attr_get_name(&self, attr: AttrHandle) -> Result<&str, DatasetError>;
	fn attr_get_type(&self, attr: AttrHandle) -> Result<StorageDataStub, DatasetError>;
	fn attr_get_class(&self, attr: AttrHandle) -> ClassHandle;
}
