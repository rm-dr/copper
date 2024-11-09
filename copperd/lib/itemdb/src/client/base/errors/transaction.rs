//! Errors we can encounter when operating on attributes
use thiserror::Error;

use crate::transaction::AddItemError;

/// An error we can encounter when creating an attribute
#[derive(Debug, Error)]
pub enum ApplyTransactionError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// We encountered an error while adding an item
	#[error("error while creating item")]
	AddItemError(#[from] AddItemError),

	/// A transaction action referenced the result of another transaction,
	/// but that other transaction doesn't exist or hasn't been computed yet.
	#[error("referenced an action that doesn't exist")]
	ReferencedBadAction,

	/// A transaction action referenced the result of another transaction,
	/// but that other transaction produced a `None` result
	#[error("referenced result with `None` return type")]
	ReferencedNoneResult,

	/// A transaction action referenced the result of another transaction,
	/// but that other transaction produced a result with an unexpected type.
	#[error("referenced result with unexpected type")]
	ReferencedResultWithBadType,
}
