//! The database client api

use crate::{
	transaction::Transaction, AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId,
	ClassInfo, DatasetId, DatasetInfo, ItemInfo, UserId,
};
use async_trait::async_trait;

use super::errors::{
	attribute::{AddAttributeError, DeleteAttributeError, GetAttributeError, RenameAttributeError},
	class::{AddClassError, DeleteClassError, GetClassError, RenameClassError},
	dataset::{
		AddDatasetError, DeleteDatasetError, GetDatasetError, ListDatasetsError, RenameDatasetError,
	},
	item::{CountItemsError, ListItemsError},
	transaction::ApplyTransactionError,
};

/// A generic database client
#[async_trait]
pub trait ItemdbClient
where
	Self: Send + Sync,
{
	//
	// MARK: Dataset
	//

	/// Create a new dataset
	async fn add_dataset(&self, name: &str, owner: UserId) -> Result<DatasetId, AddDatasetError>;

	/// Get dataset details
	async fn get_dataset(&self, dataset: DatasetId) -> Result<DatasetInfo, GetDatasetError>;

	/// Get all a user's datasets
	async fn list_datasets(&self, owner: UserId) -> Result<Vec<DatasetInfo>, ListDatasetsError>;

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

	async fn list_items(
		&self,
		class: ClassId,
		skip: i64,
		count: usize,
	) -> Result<Vec<ItemInfo>, ListItemsError>;

	async fn count_items(&self, class: ClassId) -> Result<i64, CountItemsError>;

	//
	// MARK: Transactions
	//

	/// Apply the given transaction the database
	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<(), ApplyTransactionError>;
}
