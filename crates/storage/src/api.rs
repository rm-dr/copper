use std::{fmt::Debug, hash::Hash};

use ufo_util::data::{PipelineData, PipelineDataType};

pub trait DatasetHandle:
	Clone + Copy + Eq + Hash + Debug + Send + Sync + PartialEq + PartialOrd + Ord
{
}

// TODO: count attrs

pub trait Dataset {
	type ClassHandle: DatasetHandle;
	type AttrHandle: DatasetHandle;
	type ItemHandle: DatasetHandle;
	type ErrorType: Debug;

	fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, Self::ErrorType>;
	fn add_item(&mut self, class: Self::ClassHandle) -> Result<Self::ItemHandle, Self::ErrorType>;
	fn add_item_with_attrs(
		&mut self,
		class: Self::ClassHandle,
		attrs: &[&PipelineData],
	) -> Result<Self::ItemHandle, Self::ErrorType>;
	fn add_attr(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
		data_type: PipelineDataType,
	) -> Result<Self::AttrHandle, Self::ErrorType>;

	fn del_class(&mut self, class: Self::ClassHandle) -> Result<(), Self::ErrorType>;
	fn del_item(&mut self, item: Self::ItemHandle) -> Result<(), Self::ErrorType>;
	fn del_attr(&mut self, attr: Self::AttrHandle) -> Result<(), Self::ErrorType>;

	fn iter_items(&self) -> impl Iterator<Item = Self::ItemHandle>;
	fn iter_classes(&self) -> impl Iterator<Item = Self::ClassHandle>;
	fn iter_attrs(&self) -> impl Iterator<Item = Self::AttrHandle>;

	fn get_class(&self, class_name: &str) -> Option<Self::ClassHandle>;
	fn get_attr(&self, attr_name: &str) -> Option<Self::AttrHandle>;

	fn item_set_attr(
		&mut self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
		data: &PipelineData,
	) -> Result<(), Self::ErrorType>;
	fn item_get_attr(
		&self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
	) -> Result<PipelineData, Self::ErrorType>;
	fn item_get_class(&self, item: Self::ItemHandle) -> Self::ClassHandle;

	fn class_set_name(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
	) -> Result<(), Self::ErrorType>;
	fn class_get_name(&self, class: Self::ClassHandle) -> &str;
	fn class_get_attrs(&self, class: Self::ClassHandle) -> impl Iterator<Item = Self::AttrHandle>;
	fn class_num_attrs(&self, class: Self::ClassHandle) -> usize;

	fn attr_set_name(&mut self, attr: Self::AttrHandle, name: &str) -> Result<(), Self::ErrorType>;
	fn attr_get_name(&self, attr: Self::AttrHandle) -> &str;
	fn attr_get_type(&self, attr: Self::AttrHandle) -> PipelineDataType;
	fn attr_get_class(&self, attr: Self::AttrHandle) -> Self::ClassHandle;
	// TODO: errors for bad attr
}
