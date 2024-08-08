mod mem;

use crate::model::{AttributeType, AttributeValue, Class, Item, ItemReader, ItemType, Uid};
pub use mem::MemStorageBackend;

pub trait StorageBackend<'a> {
	// Unique id types for each stored object
	type ClassUid: Uid;
	type AttrUid: Uid;
	type ItemUid: Uid;

	/// Make a new, empty class with the given name
	fn add_class(&mut self, name: &str) -> Option<Self::ClassUid>;

	/// Delete a document type
	fn del_class(&mut self, doc: Self::ClassUid) -> Option<()>;

	/// Get a class by id
	fn get_class(&self, doc: Self::ClassUid) -> Option<Class<Self::ClassUid, Self::AttrUid>>;

	/// Add an attribute to the given class
	fn add_attr(
		&mut self,
		doc: Self::ClassUid,
		name: &str,
		attr_type: AttributeType,
	) -> Option<Self::AttrUid>;

	/// Delete the given attribute from the given class
	fn del_attr(&mut self, doc: Self::ClassUid, attr: Self::AttrUid) -> Option<()>;

	// Add item with empty attributes
	// TODO: should return add error type
	fn add_item(
		&mut self,
		doc: Self::ClassUid,
		data_type: ItemType,
		data_read: &mut dyn ItemReader,
	) -> Option<Self::ItemUid>;

	// Remove the given item
	fn del_item(&mut self, item: Self::ItemUid) -> Option<()>;

	// Get an item by uid
	fn get_item(
		&'a self,
		item: Self::ItemUid,
	) -> Option<Item<'a, Self::ItemUid, Self::ClassUid, Self::AttrUid>>;

	// Set an attribute for an item
	fn set_item_attr(
		&mut self,
		item: Self::ItemUid,
		attr: Self::AttrUid,
		value: AttributeValue,
	) -> Option<()>;
}
