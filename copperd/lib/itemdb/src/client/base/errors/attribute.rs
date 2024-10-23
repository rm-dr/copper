//! Errors we can encounter when operating on attributes

use copper_util::names::NameError;
use std::{error::Error, fmt::Display};

/// An error we can encounter when creating an attribute
#[derive(Debug)]
pub enum AddAttributeError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried to add an attribute to a class that doesn't exist
	NoSuchClass,

	/// We tried to add an attribute with a name that is already taken
	UniqueViolation,

	/// We tried to create an attribute with an invalid name
	NameError(NameError),
}

impl Display for AddAttributeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NoSuchClass => {
				write!(f, "tried to add an attribute to a non-existing class")
			}
			Self::NameError(_) => write!(f, "invalid name"),
			Self::UniqueViolation => {
				write!(f, "this itemclass already has an attribute with this name")
			}
		}
	}
}

impl Error for AddAttributeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for AddAttributeError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when getting attribute info
#[derive(Debug)]
pub enum GetAttributeError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried to get an attribute by id, but it doesn't exist
	NotFound,
}

impl Display for GetAttributeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NotFound => write!(f, "attribute not found"),
		}
	}
}

impl Error for GetAttributeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for GetAttributeError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when renaming an attribute
#[derive(Debug)]
pub enum RenameAttributeError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried to add an attribute with a name that is already taken
	UniqueViolation,

	/// We tried to set an invalid name
	NameError(NameError),
}

impl Display for RenameAttributeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NameError(_) => write!(f, "invalid name"),
			Self::UniqueViolation => {
				write!(f, "this itemclass already has an attribute with this name")
			}
		}
	}
}

impl Error for RenameAttributeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for RenameAttributeError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when deleting an attribute
#[derive(Debug)]
pub enum DeleteAttributeError {
	/// Database error
	DbError(sqlx::Error),
}

impl Display for DeleteAttributeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeleteAttributeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
		}
	}
}

impl From<sqlx::Error> for DeleteAttributeError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}
