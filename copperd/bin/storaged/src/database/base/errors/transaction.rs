//! Errors we can encounter when operating on attributes

use copper_storaged::AddItemError;
use std::{error::Error, fmt::Display};

/// An error we can encounter when creating an attribute
#[derive(Debug)]
pub enum ApplyTransactionError {
	/// Database error
	DbError(sqlx::Error),

	/// We encountered an error while adding an item
	AddItemError(AddItemError),

	/// A transaction action referenced the result of another transaction,
	/// but that other transaction doesn't exist or hasn't been computed yet.
	ReferencedBadAction,

	/// A transaction action referenced the result of another transaction,
	/// but that other transaction produced a `None` result
	ReferencedNoneResult,

	/// A transaction action referenced the result of another transaction,
	/// but that other transaction produced a result with an unexpected type.
	ReferencedResultWithBadType,
}

impl Display for ApplyTransactionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::AddItemError(_) => write!(f, "error while creating item"),
			Self::ReferencedResultWithBadType => {
				write!(f, "referenced result with unexpected type")
			}
			Self::ReferencedBadAction => {
				write!(f, "referenced an action that doesn't exist")
			}
			Self::ReferencedNoneResult => {
				write!(f, "referenced result with `None` return type")
			}
		}
	}
}

impl Error for ApplyTransactionError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			Self::AddItemError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<AddItemError> for ApplyTransactionError {
	fn from(value: AddItemError) -> Self {
		Self::AddItemError(value)
	}
}

impl From<sqlx::Error> for ApplyTransactionError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}
