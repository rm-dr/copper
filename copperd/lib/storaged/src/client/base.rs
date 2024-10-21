use std::{
	error::Error,
	fmt::{Debug, Display},
};

use async_trait::async_trait;
use reqwest::StatusCode;

use crate::{
	ApplyTransactionApiError, AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId,
	ClassInfo, DatasetId, DatasetInfo, Transaction, UserId,
};

//
// MARK: errors
//

#[derive(Debug)]
pub enum StoragedRequestError {
	RequestError {
		error: Box<dyn Error + Sync + Send + 'static>,
	},
}

impl Display for StoragedRequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::RequestError { .. } => write!(f, "Request failed"),
		}
	}
}

impl Error for StoragedRequestError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::RequestError { error } => Some(error.as_ref()),
		}
	}
}

#[derive(Debug)]
pub struct GenericRequestError {
	pub code: StatusCode,
	pub message: Option<String>,
}

impl Display for GenericRequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.message {
			Some(x) => write!(f, "error code {}: {}", self.code, x),
			None => write!(f, "error code {}", self.code),
		}
	}
}

impl Error for GenericRequestError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}
}

//
// MARK: client
//

#[async_trait]
pub trait StoragedClient: Send + Sync {
	//
	// MARK: dataset
	//

	async fn add_dataset(
		&self,
		name: &str,
		owner: UserId,
	) -> Result<Result<DatasetId, GenericRequestError>, StoragedRequestError>;

	async fn get_dataset(
		&self,
		dataset: DatasetId,
	) -> Result<Result<Option<DatasetInfo>, GenericRequestError>, StoragedRequestError>;

	async fn list_datasets(
		&self,
		owner: UserId,
	) -> Result<Result<Vec<DatasetInfo>, GenericRequestError>, StoragedRequestError>;

	async fn rename_dataset(
		&self,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError>;

	async fn delete_dataset(
		&self,
		dataset: DatasetId,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError>;

	//
	// MARK: class
	//

	async fn add_class(
		&self,
		in_dataset: DatasetId,
		name: &str,
	) -> Result<Result<ClassId, GenericRequestError>, StoragedRequestError>;

	async fn get_class(
		&self,
		class_id: ClassId,
	) -> Result<Result<Option<ClassInfo>, GenericRequestError>, StoragedRequestError>;

	async fn rename_class(
		&self,
		class: ClassId,
		new_name: &str,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError>;

	async fn del_class(
		&self,
		class: ClassId,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError>;

	//
	// MARK: attribute
	//

	async fn add_attribute(
		&self,
		in_class: ClassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<Result<AttributeId, GenericRequestError>, StoragedRequestError>;

	async fn get_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<Result<Option<AttributeInfo>, GenericRequestError>, StoragedRequestError>;

	async fn rename_attribute(
		&self,
		attribute: AttributeId,
		new_name: &str,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError>;

	async fn del_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<Result<(), GenericRequestError>, StoragedRequestError>;

	//
	// MARK: other
	//

	async fn apply_transaction(
		&self,
		transaction: Transaction,
	) -> Result<Result<(), ApplyTransactionApiError>, StoragedRequestError>;
}
