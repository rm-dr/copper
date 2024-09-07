//! The database client api

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
	data::AttrDataStub,
	errors::{
		attribute::{
			AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError,
		},
		dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
		itemclass::{
			AddItemclassError, DeleteItemclassError, GetItemclassError, RenameItemclassError,
		},
	},
	handles::{AttributeId, DatasetId, ItemclassId},
	info::{AttributeInfo, DatasetInfo, ItemclassInfo},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
/// Options we can set when creating an attribute
pub struct AttributeOptions {
	/// If true, this attribute must have a value
	pub is_not_null: bool,

	/// If true, this attribute must be unique within its column
	pub unique: bool,
}

/// A generic database client
#[async_trait]
pub trait DatabaseClient
where
	Self: Send + Sync,
{
	//
	// MARK: Dataset
	//

	/// Create a new dataset
	async fn add_dataset(&self, name: &str) -> Result<DatasetId, AddDatasetError>;

	/// Delete a dataset
	async fn get_dataset(&self, dataset: DatasetId) -> Result<DatasetInfo, GetDatasetError>;

	/// Rename a dataset
	async fn rename_dataset(
		&self,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<(), RenameDatasetError>;

	/// Delete a dataset
	async fn del_dataset(&self, dataset: DatasetId) -> Result<(), DeleteDatasetError>;

	//
	// MARK: Itemclass
	//

	/// Create a new itemclass in a dataset
	async fn add_itemclass(
		&self,
		in_dataset: DatasetId,
		name: &str,
	) -> Result<ItemclassId, AddItemclassError>;

	/// Delete an itemclass
	async fn get_itemclass(
		&self,
		itemclass: ItemclassId,
	) -> Result<ItemclassInfo, GetItemclassError>;

	/// Rename an itemclass
	async fn rename_itemclass(
		&self,
		itemclass: ItemclassId,
		new_name: &str,
	) -> Result<(), RenameItemclassError>;

	/// Delete an itemclass
	async fn del_itemclass(&self, itemclass: ItemclassId) -> Result<(), DeleteItemclassError>;

	//
	// MARK: Attribute
	//

	/// Create a new attribute in an itemclass
	async fn add_attribute(
		&self,
		in_itemclass: ItemclassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<AttributeId, AddAttributeError>;

	/// Delete an attribute
	async fn get_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<AttributeInfo, GetAttributeError>;

	/// Rename an attribute
	async fn rename_attribute(
		&self,
		attribute: AttributeId,
		new_name: &str,
	) -> Result<(), RenameAttributeError>;

	/// Delete an attribute
	async fn del_attribute(&self, attribute: AttributeId) -> Result<(), DeleteAttributeError>;
}
