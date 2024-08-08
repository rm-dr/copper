use smartstring::{LazyCompact, SmartString};
use std::{fmt::Debug, hash::Hash};
use ufo_blobstore::api::BlobStore;
use ufo_util::mime::MimeType;

use crate::{
	data::{MetaDbData, MetaDbDataStub},
	errors::MetaDbError,
};

pub struct AttributeOptions {
	pub(crate) unique: bool,
	pub(crate) not_null: bool,
}

#[allow(clippy::derivable_impls)]
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

pub trait MetaDb<BlobStoreType: BlobStore>
where
	Self: Send,
{
	fn new_blob(&mut self, mime: &MimeType) -> <BlobStoreType as BlobStore>::Writer;
	fn finish_blob(
		&mut self,
		blob: <BlobStoreType as BlobStore>::Writer,
	) -> <BlobStoreType as BlobStore>::Handle;

	fn add_class(&mut self, name: &str) -> Result<ClassHandle, MetaDbError>;
	fn add_item(
		&mut self,
		class: ClassHandle,
		attrs: Vec<(AttrHandle, MetaDbData)>,
	) -> Result<ItemHandle, MetaDbError>;
	fn add_attr(
		&mut self,
		class: ClassHandle,
		name: &str,
		data_type: MetaDbDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetaDbError>;

	fn del_class(&mut self, class: ClassHandle) -> Result<(), MetaDbError>;
	fn del_item(&mut self, item: ItemHandle) -> Result<(), MetaDbError>;
	fn del_attr(&mut self, attr: AttrHandle) -> Result<(), MetaDbError>;

	//fn iter_items(&self) -> Result<impl Iterator<Item = ItemHandle>, ()>;
	//fn iter_classes(&self) -> Result<impl Iterator<Item = ClassHandle>, ()>;
	//fn iter_attrs(&self) -> Result<impl Iterator<Item = AttrHandle>, ()>;

	fn get_class(&mut self, class_name: &str) -> Result<Option<ClassHandle>, MetaDbError>;
	fn get_attr(
		&mut self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrHandle>, MetaDbError>;

	// TODO: take &[(_, _)] instead of single data
	fn item_set_attr(&mut self, attr: AttrHandle, data: MetaDbData) -> Result<(), MetaDbError>;
	fn item_get_attr(&self, item: ItemHandle, attr: AttrHandle) -> Result<MetaDbData, MetaDbError>;
	fn item_get_class(&self, item: ItemHandle) -> Result<ClassHandle, MetaDbError>;

	fn class_set_name(&mut self, class: ClassHandle, name: &str) -> Result<(), MetaDbError>;
	fn class_get_name(&self, class: ClassHandle) -> Result<&str, MetaDbError>;

	/// Get all attributes in the given class.
	/// Returns (attr handle, attr name, attr type)
	///
	/// Attribute order MUST be consistent!
	fn class_get_attrs(
		&mut self,
		class: ClassHandle,
	) -> Result<Vec<(AttrHandle, SmartString<LazyCompact>, MetaDbDataStub)>, MetaDbError>;
	fn class_num_attrs(&self, class: ClassHandle) -> Result<usize, MetaDbError>;

	fn attr_set_name(&mut self, attr: AttrHandle, name: &str) -> Result<(), MetaDbError>;
	fn attr_get_name(&self, attr: AttrHandle) -> Result<&str, MetaDbError>;
	fn attr_get_type(&self, attr: AttrHandle) -> Result<MetaDbDataStub, MetaDbError>;
	fn attr_get_class(&self, attr: AttrHandle) -> ClassHandle;
}
