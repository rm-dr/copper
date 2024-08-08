use std::{
	collections::HashMap,
	fmt::Debug,
	hash::Hash,
	io::{Read, Seek},
};

use smartstring::{LazyCompact, SmartString};

pub trait Uid: Clone + Copy + Eq + Hash + Debug {}

// Data model
#[derive(Debug, Clone, Copy)]
pub enum AttributeType {
	String,
	Integer,
}

#[derive(Debug, Clone)]
pub struct Attribute<AttrUid: Uid> {
	pub uid: AttrUid,
	pub name: SmartString<LazyCompact>,
	pub attr_type: AttributeType,
}

#[derive(Debug, Clone)]
pub struct Class<ClassUid: Uid, AttrUid: Uid> {
	pub uid: ClassUid,
	pub name: SmartString<LazyCompact>,
	pub attributes: Vec<AttrUid>,
}

#[derive(Debug, Clone)]
pub struct Dataset<ClassUid: Uid, AttrUid: Uid> {
	classes: Vec<Class<ClassUid, AttrUid>>,
}

// Actual data
// TODO: ONE data type
#[derive(Debug, Clone)]
pub enum AttributeValue {
	String(String),
	Integer(u32),
}

#[derive(Debug, Clone)]
pub struct ClassInstance<ClassUid: Uid, AttrUid: Uid> {
	pub class: ClassUid,
	pub values: HashMap<AttrUid, Option<AttributeValue>>,
}

#[derive(Debug, Copy, Clone)]
pub enum ItemType {
	Binary,
	Text,
	Audio(AudioItemType),
	//Pdf,
	//Image(ImageItemType),
	//Video(VideoItemType),
}

#[derive(Debug, Copy, Clone)]
pub enum AudioItemType {
	Mp3,
	Flac,
	//Ogg,
}

pub trait ItemReader<'a>: Read + Seek + 'a {}

pub struct Item<'a, ItemUid: Uid, ClassUid: Uid, AttrUid: Uid> {
	pub uid: ItemUid,
	pub data_type: ItemType,
	pub data: Box<dyn ItemReader<'a>>,
	pub class: ClassInstance<ClassUid, AttrUid>,
}

impl<ItemUid: Uid, ClassUid: Uid, AttrUid: Uid> Debug for Item<'_, ItemUid, ClassUid, AttrUid> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Item")
			.field("uid", &self.uid)
			.field("class", &self.class)
			.finish()
	}
}
