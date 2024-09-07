use std::collections::BTreeMap;

use async_trait::async_trait;
use data::{DatasetData, DatasetDataStub};
use errors::{
	attribute::{AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError},
	dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
	itemclass::{AddItemclassError, DeleteItemclassError, GetItemclassError, RenameItemclassError},
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

pub mod data;
pub mod errors;
pub mod handles;

use handles::{AttributeHandle, DatasetHandle, ItemIdx, ItemclassHandle};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AttributeOptions {
	pub unique: bool,
	pub is_not_null: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for AttributeOptions {
	fn default() -> Self {
		Self {
			unique: false,
			is_not_null: false,
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

	pub fn is_not_null(mut self, is_not_null: bool) -> Self {
		self.is_not_null = is_not_null;
		self
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DatasetInfo {
	#[schema(value_type = u32)]
	pub handle: DatasetHandle,

	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ItemclassInfo {
	#[schema(value_type = u32)]
	pub handle: ItemclassHandle,

	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributeInfo {
	#[schema(value_type = u32)]
	pub handle: AttributeHandle,

	#[schema(value_type = u32)]
	pub itemclass: ItemclassHandle,

	pub order: u32,

	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	pub data_type: DatasetDataStub,

	pub is_unique: bool,

	pub is_not_null: bool,
}

#[derive(Debug, Clone)]
pub struct ItemData {
	pub handle: ItemIdx,

	/// The attributes of this item.
	pub attrs: BTreeMap<AttributeHandle, DatasetData>,
}

#[async_trait]
pub trait DatabaseClient
where
	Self: Send + Sync,
{
	//
	// MARK: Dataset
	//

	/// Create a new dataset
	async fn add_dataset(&self, name: &str) -> Result<DatasetHandle, AddDatasetError>;

	/// Delete a dataset
	async fn get_dataset(&self, dataset: DatasetHandle) -> Result<DatasetInfo, GetDatasetError>;

	/// Rename a dataset
	async fn rename_dataset(
		&self,
		dataset: DatasetHandle,
		new_name: &str,
	) -> Result<(), RenameDatasetError>;

	/// Delete a dataset
	async fn del_dataset(&self, dataset: DatasetHandle) -> Result<(), DeleteDatasetError>;

	//
	// MARK: Itemclass
	//

	/// Create a new itemclass in a dataset
	async fn add_itemclass(
		&self,
		in_dataset: DatasetHandle,
		name: &str,
	) -> Result<ItemclassHandle, AddItemclassError>;

	/// Delete an itemclass
	async fn get_itemclass(
		&self,
		itemclass: ItemclassHandle,
	) -> Result<ItemclassInfo, GetItemclassError>;

	/// Rename an itemclass
	async fn rename_itemclass(
		&self,
		itemclass: ItemclassHandle,
		new_name: &str,
	) -> Result<(), RenameItemclassError>;

	/// Delete an itemclass
	async fn del_itemclass(&self, itemclass: ItemclassHandle) -> Result<(), DeleteItemclassError>;

	//
	// MARK: Attribute
	//

	/// Create a new attribute in an itemclass
	async fn add_attribute(
		&self,
		in_itemclass: ItemclassHandle,
		name: &str,
		with_type: DatasetDataStub,
		options: AttributeOptions,
	) -> Result<AttributeHandle, AddAttributeError>;

	/// Delete an attribute
	async fn get_attribute(
		&self,
		attribute: AttributeHandle,
	) -> Result<AttributeInfo, GetAttributeError>;

	/// Rename an attribute
	async fn rename_attribute(
		&self,
		attribute: AttributeHandle,
		new_name: &str,
	) -> Result<(), RenameAttributeError>;

	/// Delete an attribute
	async fn del_attribute(&self, attribute: AttributeHandle) -> Result<(), DeleteAttributeError>;
}
