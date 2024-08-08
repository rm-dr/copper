use std::{fmt::Debug, hash::Hash};

use crate::{StorageData, StorageDataType};

pub struct AttributeOptions {
	pub(crate) unique: bool,
}

impl Default for AttributeOptions {
	fn default() -> Self {
		Self { unique: false }
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemHandle {
	id: usize,
}

impl From<ItemHandle> for usize {
	fn from(value: ItemHandle) -> Self {
		value.id
	}
}

impl From<usize> for ItemHandle {
	fn from(value: usize) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClassHandle {
	id: usize,
}

impl From<ClassHandle> for usize {
	fn from(value: ClassHandle) -> Self {
		value.id
	}
}

impl From<usize> for ClassHandle {
	fn from(value: usize) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttrHandle {
	id: usize,
}

impl From<AttrHandle> for usize {
	fn from(value: AttrHandle) -> Self {
		value.id
	}
}

impl From<usize> for AttrHandle {
	fn from(value: usize) -> Self {
		Self { id: value }
	}
}

// TODO: better db backend. EAV is slow.
pub trait Dataset {
	fn add_class(&mut self, name: &str) -> Result<ClassHandle, ()>;
	fn add_item(&mut self, class: ClassHandle) -> Result<ItemHandle, ()>;
	fn add_item_with_attrs(
		&mut self,
		class: ClassHandle,
		attrs: &[&StorageData],
	) -> Result<ItemHandle, ()>;
	fn add_attr(
		&mut self,
		class: ClassHandle,
		name: &str,
		data_type: StorageDataType,
		options: AttributeOptions,
	) -> Result<AttrHandle, ()>;

	fn del_class(&mut self, class: ClassHandle) -> Result<(), ()>;
	fn del_item(&mut self, item: ItemHandle) -> Result<(), ()>;
	fn del_attr(&mut self, attr: AttrHandle) -> Result<(), ()>;

	//fn iter_items(&self) -> Result<impl Iterator<Item = ItemHandle>, ()>;
	//fn iter_classes(&self) -> Result<impl Iterator<Item = ClassHandle>, ()>;
	//fn iter_attrs(&self) -> Result<impl Iterator<Item = AttrHandle>, ()>;

	fn get_class(&self, class_name: &str) -> Result<Option<ClassHandle>, ()>;
	fn get_attr(&self, attr_name: &str) -> Result<Option<AttrHandle>, ()>;

	fn item_set_attr(
		&mut self,
		item: ItemHandle,
		attr: AttrHandle,
		data: &StorageData,
	) -> Result<(), ()>;
	fn item_get_attr(&self, item: ItemHandle, attr: AttrHandle) -> Result<StorageData, ()>;
	fn item_get_class(&self, item: ItemHandle) -> Result<ClassHandle, ()>;

	fn class_set_name(&mut self, class: ClassHandle, name: &str) -> Result<(), ()>;
	fn class_get_name(&self, class: ClassHandle) -> Result<&str, ()>;
	//fn class_get_attrs(&self, class: ClassHandle) -> Result<impl Iterator<Item = AttrHandle>, ()>;
	fn class_num_attrs(&self, class: ClassHandle) -> Result<usize, ()>;

	fn attr_set_name(&mut self, attr: AttrHandle, name: &str) -> Result<(), ()>;
	fn attr_get_name(&self, attr: AttrHandle) -> Result<&str, ()>;
	fn attr_get_type(&self, attr: AttrHandle) -> Result<StorageDataType, ()>;
	fn attr_get_class(&self, attr: AttrHandle) -> ClassHandle;
}
