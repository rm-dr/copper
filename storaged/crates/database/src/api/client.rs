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
		class::{AddClassError, DeleteClassError, GetClassError, RenameClassError},
		dataset::{AddDatasetError, DeleteDatasetError, GetDatasetError, RenameDatasetError},
		item::{DeleteItemError, GetItemError},
		transaction::ApplyTransactionError,
	},
	handles::{AttributeId, ClassId, DatasetId, ItemId},
	info::{AttributeInfo, ClassInfo, DatasetInfo, ItemInfo},
	transaction::Transaction,
};

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
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

	/// Get dataset details
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
	// MARK: Class
	//

	/// Create a new class in a dataset
	async fn add_class(&self, in_dataset: DatasetId, name: &str) -> Result<ClassId, AddClassError>;

	/// Get class details
	async fn get_class(&self, class: ClassId) -> Result<ClassInfo, GetClassError>;

	/// Rename a class
	async fn rename_class(&self, class: ClassId, new_name: &str) -> Result<(), RenameClassError>;

	/// Delete a class
	async fn del_class(&self, class: ClassId) -> Result<(), DeleteClassError>;

	//
	// MARK: Attribute
	//

	/// Create a new attribute in a class
	async fn add_attribute(
		&self,
		in_class: ClassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<AttributeId, AddAttributeError>;

	/// Get attribute details
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

	//
	// MARK: Item
	//

	/// Get item details
	async fn get_item(&self, item: ItemId) -> Result<ItemInfo, GetItemError>;

	/// Delete an item
	async fn del_item(&self, item: ItemId) -> Result<(), DeleteItemError>;

	//
	// MARK: Transactions
	//

	/// Apply the given transaction the database
	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<(), ApplyTransactionError>;
}
