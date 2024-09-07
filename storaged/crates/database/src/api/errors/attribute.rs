//! Errors we can encounter when operating on attributes

use std::{error::Error, fmt::Display};

/// An error we can encounter when creating an attribute
#[derive(Debug)]
pub enum AddAttributeError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to add an attribute to a class that doesn't exist
	NoSuchClass,
}

impl Display for AddAttributeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NoSuchClass => {
				write!(f, "tried to add an attribute to a non-existing class")
			}
		}
	}
}

impl Error for AddAttributeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

/// An error we can encounter when getting attribute info
#[derive(Debug)]
pub enum GetAttributeError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

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
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

/// An error we can encounter when renaming an attibute
#[derive(Debug)]
pub enum RenameAttributeError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for RenameAttributeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for RenameAttributeError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}

/// An error we can encounter when deleting an attribute
#[derive(Debug)]
pub enum DeleteAttributeError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
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
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
