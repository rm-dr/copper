use std::collections::HashMap;
use ufo_pipeline::data::{PipelineData, PipelineDataType};

use super::api::{Dataset, DatasetHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemItemIdx(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemClassIdx(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemAttrIdx(u32);
impl DatasetHandle for MemItemIdx {}
impl DatasetHandle for MemClassIdx {}
impl DatasetHandle for MemAttrIdx {}

struct MemAttr {
	name: String,
	class: MemClassIdx,
	data_type: PipelineDataType,
}

struct MemClass {
	name: String,
}

struct MemItem {
	class: MemClassIdx,
	data: HashMap<MemAttrIdx, Option<PipelineData>>,
}

pub struct MemDataset {
	id_counter: u32,

	classes: HashMap<MemClassIdx, MemClass>,
	attrs: HashMap<MemAttrIdx, MemAttr>,
	items: HashMap<MemItemIdx, MemItem>,
}

impl MemDataset {
	fn new_id_item(&mut self) -> MemItemIdx {
		let id = MemItemIdx(self.id_counter);
		self.id_counter += 1;
		return id;
	}

	fn new_id_class(&mut self) -> MemClassIdx {
		let id = MemClassIdx(self.id_counter);
		self.id_counter += 1;
		return id;
	}

	fn new_id_attr(&mut self) -> MemAttrIdx {
		let id = MemAttrIdx(self.id_counter);
		self.id_counter += 1;
		return id;
	}
}

impl Dataset for MemDataset {
	type AttrHandle = MemAttrIdx;
	type ClassHandle = MemClassIdx;
	type ItemHandle = MemItemIdx;

	fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, ()> {
		let id = self.new_id_class();
		self.classes.insert(
			id,
			MemClass {
				name: name.to_string(),
			},
		);
		return Ok(id);
	}

	fn add_attr(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
		data_type: PipelineDataType,
	) -> Result<Self::AttrHandle, ()> {
		let id = self.new_id_attr();
		self.attrs.insert(
			id,
			MemAttr {
				name: name.to_string(),
				class: class,
				data_type,
			},
		);
		return Ok(id);
	}

	fn add_item(&mut self, class: Self::ClassHandle) -> Result<Self::ItemHandle, ()> {
		let id = self.new_id_item();
		self.items.insert(
			id,
			MemItem {
				class,
				data: HashMap::new(),
			},
		);
		return Ok(id);
	}

	fn del_attr(&mut self, attr: Self::AttrHandle) -> Result<(), ()> {
		// TODO: delete all instances of this attr
		self.attrs.remove(&attr).unwrap();
		Ok(())
	}

	fn del_class(&mut self, class: Self::ClassHandle) -> Result<(), ()> {
		// TODO: remove all items with class
		self.classes.remove(&class);
		Ok(())
	}

	fn del_item(&mut self, item: Self::ItemHandle) -> Result<(), ()> {
		self.items.remove(&item);
		Ok(())
	}

	fn iter_items(&self) -> impl Iterator<Item = Self::ItemHandle> {
		self.items.keys().cloned()
	}

	fn iter_attrs(&self) -> impl Iterator<Item = Self::AttrHandle> {
		self.attrs.keys().cloned()
	}

	fn iter_classes(&self) -> impl Iterator<Item = Self::ClassHandle> {
		self.classes.keys().cloned()
	}

	fn item_get_attr(
		&self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
	) -> Result<Option<PipelineData>, ()> {
		Ok(self
			.items
			.get(&item)
			.unwrap()
			.data
			.get(&attr)
			.unwrap()
			.clone())
	}

	fn item_get_class(&self, item: Self::ItemHandle) -> Self::ClassHandle {
		self.items.get(&item).unwrap().class
	}

	fn item_set_attr(
		&mut self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
		data: Option<&PipelineData>,
	) -> Result<(), ()> {
		*self
			.items
			.get_mut(&item)
			.unwrap()
			.data
			.get_mut(&attr)
			.unwrap() = data.cloned();
		Ok(())
	}

	fn class_set_name(&mut self, class: Self::ClassHandle, name: &str) -> Result<(), ()> {
		self.classes.get_mut(&class).unwrap().name = name.to_string();
		Ok(())
	}

	fn class_get_name(&self, class: Self::ClassHandle) -> &str {
		&self.classes.get(&class).unwrap().name
	}

	fn class_get_attrs(&self, class: Self::ClassHandle) -> impl Iterator<Item = Self::AttrHandle> {
		self.attrs
			.iter()
			.filter_map(move |(id, attr)| if attr.class == class { Some(*id) } else { None })
	}

	fn attr_set_name(&mut self, attr: Self::AttrHandle, name: &str) -> Result<(), ()> {
		self.attrs.get_mut(&attr).unwrap().name = name.to_string();
		Ok(())
	}

	fn attr_get_name(&self, attr: Self::AttrHandle) -> &str {
		&self.attrs.get(&attr).unwrap().name
	}

	fn attr_get_type(&self, attr: Self::AttrHandle) -> PipelineDataType {
		self.attrs.get(&attr).unwrap().data_type
	}
}