//! Errors we can encounter when operating on classes

use std::{error::Error, fmt::Display};

/// An error we can encounter when listing items
#[derive(Debug)]
pub enum ListItemsError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried get items from a class that doesn't exist
	ClassNotFound,
}

impl Display for ListItemsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::ClassNotFound => write!(f, "class not found"),
		}
	}
}

impl Error for ListItemsError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for ListItemsError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when counting
#[derive(Debug)]
pub enum CountItemsError {
	/// Database error
	DbError(sqlx::Error),

	/// We tried count items in a class that doesn't exist
	ClassNotFound,
}

impl Display for CountItemsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::ClassNotFound => write!(f, "class not found"),
		}
	}
}

impl Error for CountItemsError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for CountItemsError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}
