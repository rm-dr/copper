//! Errors we can encounter when operating on attributes

use copper_util::names::NameError;
use thiserror::Error;

/// An error we can encounter when creating an attribute
#[derive(Debug, Error)]
pub enum AddAttributeError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to add an attribute to a class that doesn't exist
	#[error("tried to add an attribute to a non-existing class")]
	NoSuchClass,

	/// We tried to add an attribute with a name that is already taken
	#[error("this itemclass already has an attribute with this name")]
	UniqueViolation,

	/// We tried to create an attribute with an invalid name
	#[error("invalid name")]
	NameError(#[from] NameError),

	#[error("tried to create a `not null` attribute that would implicitly create null attributes")]
	CreatedNotNullWhenItemsExist,
}

/// An error we can encounter when getting attribute info
#[derive(Debug, Error)]
pub enum GetAttributeError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to get an attribute by id, but it doesn't exist
	#[error("attribute not found")]
	NotFound,
}

/// An error we can encounter when renaming an attribute
#[derive(Debug, Error)]
pub enum RenameAttributeError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried to add an attribute with a name that is already taken
	#[error("this itemclass already has an attribute with this name")]
	UniqueViolation,

	/// We tried to set an invalid name
	#[error("invalid name")]
	NameError(NameError),
}

/// An error we can encounter when deleting an attribute
#[derive(Debug, Error)]
pub enum DeleteAttributeError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}
