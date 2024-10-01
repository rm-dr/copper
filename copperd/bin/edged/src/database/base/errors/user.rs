//! Errors we can encounter when operating on datasets

use copper_util::names::NameError;
use std::{error::Error, fmt::Display};

/// An error we can encounter when creating a user
#[derive(Debug)]
pub enum AddUserError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// A user with this email already exists
	UniqueEmailViolation,

	/// We tried to create a user with an invalid name
	NameError(NameError),
	// TODO: bademail & badpassword
}

impl Display for AddUserError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::UniqueEmailViolation => write!(f, "a user with this email already exists"),
			Self::NameError(_) => write!(f, "invalid user name"),
		}
	}
}

impl Error for AddUserError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

/// An error we can encounter when getting a user
#[derive(Debug)]
pub enum GetUserError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for GetUserError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for GetUserError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}

/// An error we can encounter when updating a user
#[derive(Debug)]
pub enum UpdateUserError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// A user with this email already exists
	UniqueEmailViolation,

	/// We tried to set an invalid name
	NameError(NameError),
}

impl Display for UpdateUserError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UniqueEmailViolation => write!(f, "a user with this email already exists"),
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NameError(_) => write!(f, "invalid user name"),
		}
	}
}

impl Error for UpdateUserError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

/// An error we can encounter when deleting a user
#[derive(Debug)]
pub enum DeleteUserError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for DeleteUserError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeleteUserError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
