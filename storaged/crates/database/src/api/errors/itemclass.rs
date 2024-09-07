//! Errors we can encounter when operating on itemclasses

use std::{error::Error, fmt::Display};

/// An error we can encounter when creating an itemclass
#[derive(Debug)]
pub enum AddItemclassError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to add an itemclass to a dataset that doesn't exist
	NoSuchDataset,
}

impl Display for AddItemclassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NoSuchDataset => write!(f, "tried to add an itemclass to a non-existing dataset"),
		}
	}
}

impl Error for AddItemclassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

/// An error we can encounter when getting itemclass info
#[derive(Debug)]
pub enum GetItemclassError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to get an itemclass by id, but it doesn't exist
	NotFound,
}

impl Display for GetItemclassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NotFound => write!(f, "itemclass not found"),
		}
	}
}

impl Error for GetItemclassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

/// An error we can encounter when renaming an itemclass
#[derive(Debug)]
pub enum RenameItemclassError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for RenameItemclassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for RenameItemclassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}

/// An error we can encounter when deleting an itemclass
#[derive(Debug)]
pub enum DeleteItemclassError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for DeleteItemclassError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeleteItemclassError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
