//! Errors we can encounter when operating on items

use std::{error::Error, fmt::Display};

/// An error we can encounter when creating a item
#[derive(Debug)]
pub enum AddItemError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to add an item to a class that doesn't exist
	NoSuchClass,

	/// We tried to create an item that contains an
	/// attribute that doesn't exist
	BadAttribute,

	/// We tried to create an item that contains an
	/// attribute from another itemclass
	ForeignAttribute,
}

impl Display for AddItemError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NoSuchClass => write!(f, "tried to add an item to a class that doesn't exist"),
			Self::BadAttribute => {
				write!(f, "tried to create an item an attribute that doesn't exist")
			}
			Self::ForeignAttribute => write!(f, "tried to create an item with a foreign attribute"),
		}
	}
}

impl Error for AddItemError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

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
