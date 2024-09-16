//! Errors we can encounter when operating on attributes

use std::{error::Error, fmt::Display};

/// An error we can encounter when creating an attribute
#[derive(Debug)]
pub enum ApplyTransactionError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

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
			Self::DbError(x) => Some(x.as_ref()),
			Self::AddItemError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<AddItemError> for ApplyTransactionError {
	fn from(value: AddItemError) -> Self {
		match value {
			AddItemError::DbError(e) => Self::DbError(e),
			x => Self::AddItemError(x),
		}
	}
}

/// An error we can encounter when creating an item
#[derive(Debug)]
pub enum AddItemError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to add an item to a class that doesn't exist
	NoSuchClass,

	/// We tried to create an item that contains an
	/// attribute that doesn't exist
	BadAttribute,

	/// We tried to create an item,
	/// but provided multiple values for one attribute
	RepeatedAttribute,

	/// We tried to assign data to an attribute,
	/// but that data has the wrong type
	AttributeDataTypeMismatch,

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
			Self::RepeatedAttribute => {
				write!(f, "multiple values were provided for one attribute")
			}
			Self::AttributeDataTypeMismatch => {
				write!(
					f,
					"tried to assign data to an attribute, but type doesn't match"
				)
			}
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
