//! Errors we can encounter when operating on classes

use copper_util::names::NameError;
use std::{error::Error, fmt::Display};

/// An error we can encounter when creating a class
#[derive(Debug)]
pub enum AddClassError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried to add a class to a dataset that doesn't exist
	NoSuchDataset,

	/// We tried to add a class, but its dataset already has a class with that name
	UniqueViolation,

	/// We tried to create a class with an invalid name
	NameError(NameError),
}

impl Display for AddClassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NoSuchDataset => write!(f, "tried to add a class to a non-existing dataset"),
			Self::NameError(_) => write!(f, "invalid name"),
			Self::UniqueViolation => write!(f, "this dataset already has a class with this name"),
		}
	}
}

impl Error for AddClassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for AddClassError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when getting class info
#[derive(Debug)]
pub enum GetClassError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried to get a class by id, but it doesn't exist
	NotFound,
}

impl Display for GetClassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NotFound => write!(f, "class not found"),
		}
	}
}

impl Error for GetClassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for GetClassError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when renaming a class
#[derive(Debug)]
pub enum RenameClassError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried to set an invalid name
	NameError(NameError),

	/// We tried to rename a class to a name that is already taken
	UniqueViolation,
}

impl Display for RenameClassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NameError(_) => write!(f, "invalid name"),
			Self::UniqueViolation => write!(f, "this dataset already has a class with this name"),
		}
	}
}

impl Error for RenameClassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for RenameClassError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when deleting a class
#[derive(Debug)]
pub enum DeleteClassError {
	/// Database error
	DbError(sqlx::Error),
}

impl Display for DeleteClassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeleteClassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
		}
	}
}

impl From<sqlx::Error> for DeleteClassError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}
