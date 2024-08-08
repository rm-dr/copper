use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use crate::{
	data::{MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::{AttrHandle, ClassHandle, ItemIdx},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AttributeOptions {
	pub unique: bool,
}

#[allow(clippy::derivable_impls)]
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

#[derive(Debug, Clone)]
pub struct ItemData {
	pub handle: ItemIdx,
	// Attrs are in the same order as class_get_attrs
	// TODO: this is important, document it!
	pub attrs: Vec<MetastoreData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttrInfo {
	#[schema(value_type = u32)]
	pub handle: AttrHandle,

	#[schema(value_type = u32)]
	pub class: ClassHandle,

	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	pub data_type: MetastoreDataStub,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClassInfo {
	#[schema(value_type = u32)]
	pub handle: ClassHandle,

	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,
}

#[allow(async_fn_in_trait)]
pub trait Metastore
where
	Self: Send + Sync,
{
	async fn add_class(&self, name: &str) -> Result<ClassHandle, MetastoreError>;
	async fn add_item(
		&self,
		class: ClassHandle,
		attrs: Vec<(AttrHandle, MetastoreData)>,
	) -> Result<ItemIdx, MetastoreError>;
	async fn add_attr(
		&self,
		class: ClassHandle,
		name: &str,
		data_type: MetastoreDataStub,
		options: AttributeOptions,
	) -> Result<AttrHandle, MetastoreError>;

	async fn del_class(&self, class: ClassHandle) -> Result<(), MetastoreError>;
	async fn del_item(&self, item: ItemIdx) -> Result<(), MetastoreError>;
	async fn del_attr(&self, attr: AttrHandle) -> Result<(), MetastoreError>;

	async fn get_all_classes(&self) -> Result<Vec<ClassInfo>, MetastoreError>;
	async fn get_all_attrs(&self) -> Result<Vec<AttrInfo>, MetastoreError>;

	async fn get_class_by_name(
		&self,
		class_name: &str,
	) -> Result<Option<ClassInfo>, MetastoreError>;
	async fn get_class(&self, class: ClassHandle) -> Result<ClassInfo, MetastoreError>;

	async fn get_attr_by_name(
		&self,
		class: ClassHandle,
		attr_name: &str,
	) -> Result<Option<AttrInfo>, MetastoreError>;
	async fn get_attr(&self, attr: AttrHandle) -> Result<AttrInfo, MetastoreError>;

	async fn get_item_attr(
		&self,
		attr: AttrHandle,
		item: ItemIdx,
	) -> Result<MetastoreData, MetastoreError>;

	async fn class_set_name(&self, class: ClassHandle, name: &str) -> Result<(), MetastoreError>;

	/// Get all classes that store references to items in this class.
	/// Returns class handles and names, and INCLUDES this class if it references itself.
	async fn class_get_backlinks(
		&self,
		class: ClassHandle,
	) -> Result<Vec<ClassInfo>, MetastoreError>;

	/// Get all attributes in the given class.
	/// Returns (attr handle, attr name, attr type)
	///
	/// Attribute order MUST be consistent!
	async fn class_get_attrs(&self, class: ClassHandle) -> Result<Vec<AttrInfo>, MetastoreError>;
	async fn class_num_attrs(&self, class: ClassHandle) -> Result<usize, MetastoreError>;

	async fn attr_set_name(&self, attr: AttrHandle, name: &str) -> Result<(), MetastoreError>;

	// TODO: clean this up. What does this method do?
	// Document the fact that attrhandle implies a class.
	async fn find_item_with_attr(
		&self,
		attr: AttrHandle,
		attr_value: MetastoreData,
	) -> Result<Option<ItemIdx>, MetastoreError>;

	async fn get_items(
		&self,
		class: ClassHandle,
		page_size: u32,
		start_at: u32,
	) -> Result<Vec<ItemData>, MetastoreError>;
}
