use std::{fmt::Debug, hash::Hash};
use ufo_util::data::{PipelineData, PipelineDataType};

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

pub trait DatasetHandle:
	Clone + Copy + Eq + Hash + Debug + Send + Sync + PartialEq + PartialOrd + Ord
{
}

// TODO: better db backend. EAV is slow.
// TODO: count attrs
// TODO: why do we need `async_fn_in_trait`?
#[allow(async_fn_in_trait)]
pub trait Dataset {
	type ClassHandle: DatasetHandle;
	type AttrHandle: DatasetHandle;
	type ItemHandle: DatasetHandle;
	type ErrorType: Debug;

	async fn add_class(&mut self, name: &str) -> Result<Self::ClassHandle, Self::ErrorType>;
	async fn add_item(
		&mut self,
		class: Self::ClassHandle,
	) -> Result<Self::ItemHandle, Self::ErrorType>;
	async fn add_item_with_attrs(
		&mut self,
		class: Self::ClassHandle,
		attrs: &[&PipelineData],
	) -> Result<Self::ItemHandle, Self::ErrorType>;
	async fn add_attr(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
		data_type: PipelineDataType,
		options: AttributeOptions,
	) -> Result<Self::AttrHandle, Self::ErrorType>;

	async fn del_class(&mut self, class: Self::ClassHandle) -> Result<(), Self::ErrorType>;
	async fn del_item(&mut self, item: Self::ItemHandle) -> Result<(), Self::ErrorType>;
	async fn del_attr(&mut self, attr: Self::AttrHandle) -> Result<(), Self::ErrorType>;

	async fn iter_items(&self) -> Result<impl Iterator<Item = Self::ItemHandle>, Self::ErrorType>;
	async fn iter_classes(
		&self,
	) -> Result<impl Iterator<Item = Self::ClassHandle>, Self::ErrorType>;
	async fn iter_attrs(&self) -> Result<impl Iterator<Item = Self::AttrHandle>, Self::ErrorType>;

	async fn get_class(
		&self,
		class_name: &str,
	) -> Result<Option<Self::ClassHandle>, Self::ErrorType>;
	async fn get_attr(&self, attr_name: &str) -> Result<Option<Self::AttrHandle>, Self::ErrorType>;

	async fn item_set_attr(
		&mut self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
		data: &PipelineData,
	) -> Result<(), Self::ErrorType>;
	async fn item_get_attr(
		&self,
		item: Self::ItemHandle,
		attr: Self::AttrHandle,
	) -> Result<PipelineData, Self::ErrorType>;
	async fn item_get_class(
		&self,
		item: Self::ItemHandle,
	) -> Result<Self::ClassHandle, Self::ErrorType>;

	async fn class_set_name(
		&mut self,
		class: Self::ClassHandle,
		name: &str,
	) -> Result<(), Self::ErrorType>;
	async fn class_get_name(&self, class: Self::ClassHandle) -> Result<&str, Self::ErrorType>;
	async fn class_get_attrs(
		&self,
		class: Self::ClassHandle,
	) -> Result<impl Iterator<Item = Self::AttrHandle>, Self::ErrorType>;
	async fn class_num_attrs(&self, class: Self::ClassHandle) -> Result<usize, Self::ErrorType>;

	async fn attr_set_name(
		&mut self,
		attr: Self::AttrHandle,
		name: &str,
	) -> Result<(), Self::ErrorType>;
	async fn attr_get_name(&self, attr: Self::AttrHandle) -> Result<&str, Self::ErrorType>;
	async fn attr_get_type(
		&self,
		attr: Self::AttrHandle,
	) -> Result<PipelineDataType, Self::ErrorType>;
	async fn attr_get_class(&self, attr: Self::AttrHandle) -> Self::ClassHandle;
	// TODO: errors for bad attr
}
