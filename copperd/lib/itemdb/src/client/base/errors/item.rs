//! Errors we can encounter when operating on classes
use thiserror::Error;

/// An error we can encounter when listing items
#[derive(Debug, Error)]
pub enum ListItemsError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried get items from a class that doesn't exist
	#[error("class not found")]
	ClassNotFound,
}

/// An error we can encounter when counting
#[derive(Debug, Error)]
pub enum CountItemsError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We tried count items in a class that doesn't exist
	#[error("class not found")]
	ClassNotFound,
}

/// An error we can encounter when getting item info
#[derive(Debug, Error)]
pub enum GetItemError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// An item with this id doesn't exist
	#[error("item not found")]
	NotFound,
}
