//! Errors we can encounter when operating on datasets

use std::{error::Error, fmt::Display};

use storaged_util::names::NameError;

/// An error we can encounter when creating a dataset
#[derive(Debug)]
pub enum AddDatasetError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// A dataset with this name already exists
	/// TODO: scope to user
	UniqueViolation,

	/// We tried to create a dataset with an invalid name
	NameError(NameError),
}

impl Display for AddDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::UniqueViolation => write!(f, "a dataset with this name already exists"),
			Self::NameError(_) => write!(f, "invalid name"),
		}
	}
}

impl Error for AddDatasetError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

/// An error we can encounter when getting dataset info
#[derive(Debug)]
pub enum GetDatasetError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to get a dataset by id, but it doesn't exist
	NotFound,
}

impl Display for GetDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NotFound => write!(f, "dataset not found"),
		}
	}
}

impl Error for GetDatasetError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

/// An error we can encounter when renaming a dataset
#[derive(Debug)]
pub enum RenameDatasetError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// A dataset with this name already exists
	UniqueViolation,

	/// We tried to set an invalid name
	NameError(NameError),
}

impl Display for RenameDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NameError(_) => write!(f, "invalid name"),
			Self::UniqueViolation => write!(f, "a dataset with this name already exists"),
		}
	}
}

impl Error for RenameDatasetError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

/// An error we can encounter when deleting a dataset
#[derive(Debug)]
pub enum DeleteDatasetError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for DeleteDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeleteDatasetError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
