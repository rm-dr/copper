//! Errors we can encounter when operating on datasets

use copper_util::names::NameError;
use thiserror::Error;

/// An error we can encounter when creating a dataset
#[derive(Debug, Error)]
pub enum AddDatasetError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// A dataset with this name already exists
	#[error("a dataset with this name already exists")]
	UniqueViolation,

	/// We tried to create a dataset with an invalid name
	#[error("invalid name")]
	NameError(#[from] NameError),
}

/// An error we can encounter when getting dataset info
#[derive(Debug, Error)]
pub enum GetDatasetError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to get a dataset by id, but it doesn't exist
	#[error("dataset not found")]
	NotFound,
}

/// An error we can encounter when listing a user's datasets
#[derive(Debug, Error)]
pub enum ListDatasetsError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}

/// An error we can encounter when renaming a dataset
#[derive(Debug, Error)]
pub enum RenameDatasetError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// A dataset with this name already exists
	#[error("a dataset with this name already exists")]
	UniqueViolation,

	/// We tried to set an invalid name
	#[error("invalid name")]
	NameError(NameError),
}

/// An error we can encounter when deleting a dataset
#[derive(Debug, Error)]
pub enum DeleteDatasetError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}
