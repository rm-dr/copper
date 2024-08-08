use std::{fmt::Debug, hash::Hash};
use ufo_pipeline::data::{PipelineData, PipelineDataType};

pub trait DatasetHandle: Clone + Copy + Eq + Hash + Debug + Send + Sync {}

pub trait Dataset {
	type ClassHandle: DatasetHandle;
	type AttrHandle: DatasetHandle;
	type ItemHandle: DatasetHandle;

	fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, ()>;
	fn add_item(&mut self, class: Self::ClassHandle) -> Result<Self::ItemHandle, ()>;
	fn add_attr(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
		data_type: PipelineDataType,
	) -> Result<Self::AttrHandle, ()>;

	fn del_class(&mut self, class: Self::ClassHandle) -> Result<(), ()>;
	fn del_item(&mut self, item: Self::ItemHandle) -> Result<(), ()>;
	fn del_attr(&mut self, attr: Self::AttrHandle) -> Result<(), ()>;

	fn iter_items(&self) -> impl Iterator<Item = Self::ItemHandle>;
	fn iter_classes(&self) -> impl Iterator<Item = Self::ClassHandle>;
	fn iter_attrs(&self) -> impl Iterator<Item = Self::AttrHandle>;

	fn item_set_attr(
		&mut self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
		data: Option<&PipelineData>,
	) -> Result<(), ()>;
	fn item_get_attr(
		&self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
	) -> Result<Option<PipelineData>, ()>;
	fn item_get_class(&self, item: Self::ItemHandle) -> Self::ClassHandle;

	fn class_set_name(&mut self, class: Self::ClassHandle, name: &str) -> Result<(), ()>;
	fn class_get_name(&self, class: Self::ClassHandle) -> &str;
	fn class_get_attrs(&self, class: Self::ClassHandle) -> impl Iterator<Item = Self::AttrHandle>;

	fn attr_set_name(&mut self, attr: Self::AttrHandle, name: &str) -> Result<(), ()>;
	fn attr_get_name(&self, attr: Self::AttrHandle) -> &str;
	fn attr_get_type(&self, attr: Self::AttrHandle) -> PipelineDataType;
}
