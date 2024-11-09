//! Errors we can encounter when operating on datasets

use copper_util::names::NameError;
use thiserror::Error;

/// An error we can encounter when creating a user
#[derive(Debug, Error)]
pub enum AddUserError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// A user with this email already exists
	#[error("a user with this email already exists")]
	UniqueEmailViolation,

	/// We tried to create a user with an invalid name
	#[error("invalid user name")]
	NameError(#[from] NameError),
	// TODO: bademail & badpassword
}

/// An error we can encounter when getting a user
#[derive(Debug, Error)]
pub enum GetUserError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}

/// An error we can encounter when updating a user
#[derive(Debug, Error)]
pub enum UpdateUserError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// A user with this email already exists
	#[error("a user with this email already exists")]
	UniqueEmailViolation,

	/// We tried to set an invalid name
	#[error("invalid user name")]
	NameError(#[from] NameError),
}

/// An error we can encounter when deleting a user
#[derive(Debug, Error)]
pub enum DeleteUserError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}
