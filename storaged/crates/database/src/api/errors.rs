use std::{error::Error, fmt::Display};

use copper_util::names::NameError;
use smartstring::{LazyCompact, SmartString};

#[derive(Debug)]
pub enum MetastoreError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// We tried to delete a class, but another class stores
	/// references to its items.
	///
	/// Includes a list of class names that reference the class we tried to delete.
	/// This list will NOT include the class we tried to delete.
	DeleteClassDanglingRef(Vec<SmartString<LazyCompact>>),

	/// We were given a bad attribute handle
	BadAttrHandle,

	/// We tried to create an attr with an invalid name.
	BadAttrName(NameError),

	/// We were given a bad class handle
	BadClassHandle,

	/// We tried to create a class with an invalid name.
	BadClassName(NameError),

	/// We were given a bad item idx
	BadItemIdx,

	/// We tried to set an attribute with data of a different type
	TypeMismatch,

	/// We tried to set a non-negative number to a negative value
	NonNegativeViolated,

	/// A `unique` constraint was violated
	UniqueViolated,

	/// A `not none` constraint was violated
	NotNoneViolated,

	/// We tried to create an attribute with a name that already exists
	DuplicateAttrName(SmartString<LazyCompact>),

	/// We tried to create an class with a name that already exists
	DuplicateClassName(SmartString<LazyCompact>),
}

impl Display for MetastoreError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::DeleteClassDanglingRef(_) => {
				write!(f, "Cannot delete class, would create dangling references")
			}
			Self::BadAttrHandle => write!(f, "BadAttrHandle"),
			Self::BadClassHandle => write!(f, "BadClassHandle"),
			Self::BadClassName(_) => write!(f, "BadClassName"),
			Self::BadAttrName(_) => write!(f, "BadAttrName"),
			Self::NonNegativeViolated => write!(f, "NonnegativeViolated"),
			Self::BadItemIdx => write!(f, "BadItemIdx"),
			Self::TypeMismatch => write!(f, "TypeMismatch"),
			Self::UniqueViolated => write!(f, "UniqueViolated"),
			Self::NotNoneViolated => write!(f, "NotNoneViolated"),
			Self::DuplicateAttrName(x) => write!(f, "DuplicateAttrName: `{x}`"),
			Self::DuplicateClassName(x) => write!(f, "DuplicateClassName: `{x}`"),
		}
	}
}

impl Error for MetastoreError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::BadAttrName(x) => Some(x),
			Self::BadClassName(x) => Some(x),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum BlobstoreError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// Filesystem I/O error
	IOError(std::io::Error),

	/// This blob doesn't exist
	InvalidBlobHandle,
}

impl Display for BlobstoreError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::IOError(_) => write!(f, "I/O error"),
			Self::InvalidBlobHandle => write!(f, "Invalid blob handle"),
		}
	}
}

impl Error for BlobstoreError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::IOError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<std::io::Error> for BlobstoreError {
	fn from(value: std::io::Error) -> Self {
		Self::IOError(value)
	}
}