use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use copper_edged::UserId;
use reqwest::StatusCode;

use crate::{
	AttrDataStub, AttributeId, AttributeInfo, AttributeOptions, ClassId, ClassInfo, DatasetId,
	DatasetInfo, Transaction,
};

#[derive(Debug)]
pub enum StoragedRequestError {
	GenericHttp {
		code: StatusCode,
		message: Option<String>,
	},
	Other {
		error: Box<dyn Error + Sync + Send + 'static>,
	},
}

impl Display for StoragedRequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::GenericHttp { code, message } => {
				if let Some(m) = message {
					write!(f, "Request failed with code {code}: {m}")
				} else {
					write!(f, "Request failed with code {code}")
				}
			}
			Self::Other { .. } => write!(f, "request failed"),
		}
	}
}

impl Error for StoragedRequestError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other { error } => Some(error.as_ref()),
			_ => None,
		}
	}
}

#[async_trait]
pub trait StoragedClient: Send + Sync {
	//
	// MARK: dataset
	//

	async fn add_dataset(
		&self,
		name: &str,
		owner: UserId,
	) -> Result<DatasetId, StoragedRequestError>;

	async fn get_dataset(
		&self,
		dataset: DatasetId,
	) -> Result<Option<DatasetInfo>, StoragedRequestError>;

	async fn rename_dataset(
		&self,
		dataset: DatasetId,
		new_name: &str,
	) -> Result<(), StoragedRequestError>;

	async fn delete_dataset(&self, dataset: DatasetId) -> Result<(), StoragedRequestError>;

	//
	// MARK: class
	//

	async fn add_class(
		&self,
		in_dataset: DatasetId,
		name: &str,
	) -> Result<ClassId, StoragedRequestError>;

	async fn get_class(&self, class_id: ClassId)
		-> Result<Option<ClassInfo>, StoragedRequestError>;

	async fn rename_class(
		&self,
		class: ClassId,
		new_name: &str,
	) -> Result<(), StoragedRequestError>;

	async fn del_class(&self, class: ClassId) -> Result<(), StoragedRequestError>;

	//
	// MARK: attribute
	//

	async fn add_attribute(
		&self,
		in_class: ClassId,
		name: &str,
		with_type: AttrDataStub,
		options: AttributeOptions,
	) -> Result<AttributeId, StoragedRequestError>;

	async fn get_attribute(
		&self,
		attribute: AttributeId,
	) -> Result<Option<AttributeInfo>, StoragedRequestError>;

	async fn rename_attribute(
		&self,
		attribute: AttributeId,
		new_name: &str,
	) -> Result<(), StoragedRequestError>;

	async fn del_attribute(&self, attribute: AttributeId) -> Result<(), StoragedRequestError>;

	//
	// MARK: other
	//

	async fn apply_transaction(&self, transaction: Transaction)
		-> Result<(), StoragedRequestError>;
}
