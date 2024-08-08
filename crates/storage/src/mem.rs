use std::{collections::HashMap, hash::Hash};

use ufo_util::data::{PipelineData, PipelineDataType};

use super::api::{Dataset, DatasetHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemItemIdx(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemClassIdx(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemAttrIdx(u32);
impl DatasetHandle for MemItemIdx {}
impl DatasetHandle for MemClassIdx {}
impl DatasetHandle for MemAttrIdx {}

#[derive(Debug)]
struct MemAttr {
	name: String,
	class: MemClassIdx,
	data_type: PipelineDataType,
}

#[derive(Debug)]
struct MemClass {
	name: String,
}

#[derive(Debug)]
struct MemItem {
	class: MemClassIdx,
	data: HashMap<MemAttrIdx, PipelineData>,
}

#[derive(Debug)]
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

	pub fn new() -> Self {
		Self {
			id_counter: 0,
			classes: HashMap::new(),
			attrs: HashMap::new(),
			items: HashMap::new(),
		}
	}
}

impl Default for MemDataset {
	fn default() -> Self {
		Self::new()
	}
}

impl Dataset for MemDataset {
	type AttrHandle = MemAttrIdx;
	type ClassHandle = MemClassIdx;
	type ItemHandle = MemItemIdx;
	type ErrorType = ();

	fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, Self::ErrorType> {
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
	) -> Result<Self::AttrHandle, Self::ErrorType> {
		let id = self.new_id_attr();
		self.attrs.insert(
			id,
			MemAttr {
				name: name.to_string(),
				class,
				data_type,
			},
		);
		return Ok(id);
	}

	fn add_item(&mut self, class: Self::ClassHandle) -> Result<Self::ItemHandle, Self::ErrorType> {
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

	fn add_item_with_attrs(
		&mut self,
		class: Self::ClassHandle,
		attrs: &[&PipelineData],
	) -> Result<Self::ItemHandle, Self::ErrorType> {
		let mut data = HashMap::new();

		for (i, a) in self.class_get_attrs(class).enumerate() {
			data.insert(a, (*attrs.get(i).unwrap()).clone());
		}

		let id = self.new_id_item();
		self.items.insert(id, MemItem { class, data });
		return Ok(id);
	}

	fn del_attr(&mut self, attr: Self::AttrHandle) -> Result<(), Self::ErrorType> {
		// TODO: delete all instances of this attr
		self.attrs.remove(&attr).unwrap();
		Ok(())
	}

	fn del_class(&mut self, class: Self::ClassHandle) -> Result<(), Self::ErrorType> {
		// TODO: remove all items with class
		self.classes.remove(&class);
		Ok(())
	}

	fn del_item(&mut self, item: Self::ItemHandle) -> Result<(), Self::ErrorType> {
		self.items.remove(&item);
		Ok(())
	}

	fn get_attr(&self, attr_name: &str) -> Option<Self::AttrHandle> {
		self.attrs
			.iter()
			.find_map(|(x, y)| (y.name == attr_name).then_some(*x))
	}

	fn get_class(&self, class_name: &str) -> Option<Self::ClassHandle> {
		self.classes
			.iter()
			.find_map(|(x, y)| (y.name == class_name).then_some(*x))
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
	) -> Result<PipelineData, Self::ErrorType> {
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
		data: &PipelineData,
	) -> Result<(), Self::ErrorType> {
		self.items
			.get_mut(&item)
			.unwrap()
			.data
			.insert(attr, data.clone());
		Ok(())
	}

	fn class_set_name(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
	) -> Result<(), Self::ErrorType> {
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

	fn class_num_attrs(&self, class: Self::ClassHandle) -> usize {
		self.attrs
			.iter()
			.filter(move |(_, attr)| attr.class == class)
			.count()
	}

	fn attr_set_name(&mut self, attr: Self::AttrHandle, name: &str) -> Result<(), Self::ErrorType> {
		self.attrs.get_mut(&attr).unwrap().name = name.to_string();
		Ok(())
	}

	fn attr_get_name(&self, attr: Self::AttrHandle) -> &str {
		&self.attrs.get(&attr).unwrap().name
	}

	fn attr_get_type(&self, attr: Self::AttrHandle) -> PipelineDataType {
		self.attrs.get(&attr).unwrap().data_type
	}

	fn attr_get_class(&self, attr: Self::AttrHandle) -> Self::ClassHandle {
		self.attrs.get(&attr).unwrap().class
	}
}
