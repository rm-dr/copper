//! Errors we can encounter when operating on items

use std::{error::Error, fmt::Display};

/// An error we can encounter when getting item info
#[derive(Debug)]
pub enum GetItemError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to get an item by id, but it doesn't exist
	NotFound,
}

impl Display for GetItemError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NotFound => write!(f, "item not found"),
		}
	}
}

impl Error for GetItemError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

/// An error we can encounter when deleting an item
#[derive(Debug)]
pub enum DeleteItemError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for DeleteItemError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeleteItemError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
