use std::{
	collections::HashMap,
	hash::Hash,
	io::{Cursor, Read, Seek},
};

use super::{StorageBackend, Uid};
use crate::model::{
	Attribute, AttributeType, AttributeValue, Class, ClassInstance, Item, ItemReader, ItemType,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemItemUid(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemClassUid(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemAttrUid(u32);
impl Uid for MemItemUid {}
impl Uid for MemClassUid {}
impl Uid for MemAttrUid {}

struct MemItem {
	pub uid: MemItemUid,
	pub data_type: ItemType,
	pub data: Vec<u8>,
	pub class: MemClassUid,
	pub attrs: HashMap<MemAttrUid, AttributeValue>,
}

struct MemReader<'a> {
	data: Cursor<&'a Vec<u8>>,
}

impl Read for MemReader<'_> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.data.read(buf)
	}
}

impl Seek for MemReader<'_> {
	fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
		self.data.seek(pos)
	}
}

impl<'a> ItemReader<'a> for MemReader<'a> {}

pub struct MemStorageBackend {
	id_counter: u32,

	// Data structure
	classes: HashMap<MemClassUid, Class<MemClassUid, MemAttrUid>>,
	attrs: HashMap<MemAttrUid, Attribute<MemAttrUid>>,

	// Data
	items: HashMap<MemItemUid, MemItem>,
}

impl MemStorageBackend {
	pub fn new() -> Self {
		MemStorageBackend {
			id_counter: 0,
			classes: Default::default(),
			attrs: Default::default(),
			items: Default::default(),
		}
	}

	fn new_item_uid(&mut self) -> MemItemUid {
		let uid = MemItemUid(self.id_counter);
		self.id_counter += 1;
		return uid;
	}

	fn new_class_uid(&mut self) -> MemClassUid {
		let uid = MemClassUid(self.id_counter);
		self.id_counter += 1;
		return uid;
	}

	fn new_attr_uid(&mut self) -> MemAttrUid {
		let uid = MemAttrUid(self.id_counter);
		self.id_counter += 1;
		return uid;
	}
}

impl<'a> StorageBackend<'a> for MemStorageBackend {
	type ClassUid = MemClassUid;
	type AttrUid = MemAttrUid;
	type ItemUid = MemItemUid;

	fn add_class(&mut self, name: &str) -> Option<Self::ClassUid> {
		let uid = self.new_class_uid();

		self.classes.insert(
			uid,
			Class {
				uid,
				name: name.into(),
				attributes: Default::default(),
			},
		);
		return Some(uid);
	}

	fn del_class(&mut self, doc: Self::ClassUid) -> Option<()> {
		self.classes.remove(&doc);
		return Some(());
	}

	fn get_class(&self, doc: Self::ClassUid) -> Option<Class<Self::ClassUid, Self::AttrUid>> {
		let d = self.classes.get(&doc);

		d.map(|d| Class {
			uid: d.uid,
			name: d.name.clone(),
			attributes: d.attributes.clone(),
		})
	}

	fn add_attr(
		&mut self,
		doc: Self::ClassUid,
		name: &str,
		attr: AttributeType,
	) -> Option<Self::AttrUid> {
		// This class doesn't exist
		if !self.classes.contains_key(&doc) {
			return None;
		}

		let uid = self.new_attr_uid();
		let dt = self.classes.get_mut(&doc).unwrap();
		dt.attributes.push(uid);
		self.attrs.insert(
			uid,
			Attribute {
				uid,
				name: name.into(),
				attr_type: attr,
			},
		);

		return Some(uid);
	}

	fn del_attr(&mut self, _doc: Self::ClassUid, attr: Self::AttrUid) -> Option<()> {
		self.attrs.remove(&attr);
		return Some(());
	}

	fn add_item(
		&mut self,
		doc: Self::ClassUid,
		data_type: ItemType,
		data_read: &mut dyn ItemReader,
	) -> Option<Self::ItemUid> {
		let uid = self.new_item_uid();

		let mut data: Vec<u8> = Vec::new();
		data_read.read_to_end(&mut data).unwrap();

		self.items.insert(
			uid,
			MemItem {
				uid,
				data,
				data_type,
				class: doc,
				attrs: HashMap::new(),
			},
		);

		return Some(uid);
	}

	fn del_item(&mut self, item: Self::ItemUid) -> Option<()> {
		self.items.remove(&item);
		return Some(());
	}

	fn get_item(
		&'a self,
		item: Self::ItemUid,
	) -> Option<Item<'a, Self::ItemUid, Self::ClassUid, Self::AttrUid>> {
		let d = self.items.get(&item);

		d.map(|d| Item {
			uid: d.uid,
			data_type: d.data_type,
			data: Box::new(MemReader {
				data: Cursor::new(&d.data),
			}),
			class: ClassInstance {
				class: d.class,
				values: {
					let mut x = HashMap::new();
					let dt = self.classes.get(&d.class).unwrap();
					for a in &dt.attributes {
						x.insert(*a, d.attrs.get(a).cloned());
					}
					x
				},
			},
		})
	}

	fn set_item_attr(
		&mut self,
		item: Self::ItemUid,
		attr: Self::AttrUid,
		value: AttributeValue,
	) -> Option<()> {
		let it = self.items.get_mut(&item);

		// If this item does not exist, return None
		it.as_ref()?;

		let it = it.unwrap();
		let dt = self.classes.get(&it.class).unwrap();

		// If this attribute does not apply to this item
		if !dt.attributes.contains(&attr) {
			return None;
		}

		it.attrs.insert(attr, value);
		return Some(());
	}
}
