//! Errors we can encounter when operating on classes

use copper_util::names::NameError;
use thiserror::Error;

/// An error we can encounter when creating a class
#[derive(Debug, Error)]
pub enum AddClassError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to add a class to a dataset that doesn't exist
	#[error("tried to add a class to a non-existing dataset")]
	NoSuchDataset,

	/// We tried to add a class, but its dataset already has a class with that name
	#[error("this dataset already has a class with this name")]
	UniqueViolation,

	/// We tried to create a class with an invalid name
	#[error("invalid name")]
	NameError(#[from] NameError),
}

/// An error we can encounter when getting class info
#[derive(Debug, Error)]
pub enum GetClassError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to get a class by id, but it doesn't exist
	#[error("class not found")]
	NotFound,
}

/// An error we can encounter when renaming a class
#[derive(Debug, Error)]
pub enum RenameClassError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to set an invalid name
	#[error("invalid name")]
	NameError(#[from] NameError),

	/// We tried to rename a class to a name that is already taken
	#[error("this dataset already has a class with this name")]
	UniqueViolation,
}

/// An error we can encounter when deleting a class
#[derive(Debug, Error)]
pub enum DeleteClassError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}
