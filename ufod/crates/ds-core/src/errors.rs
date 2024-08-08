use std::{error::Error, fmt::Display};

use smartstring::{LazyCompact, SmartString};

#[derive(Debug)]
pub enum MetastoreError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We were given a bad attribute handle
	BadAttrHandle,

	/// We were given a bad class handle
	BadClassHandle,

	/// We tried to set an attribute with data of a different type
	TypeMismatch,

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
			Self::BadAttrHandle => write!(f, "BadAttrHandle"),
			Self::BadClassHandle => write!(f, "BadClassHandle"),
			Self::TypeMismatch => write!(f, "TypeMismatch"),
			Self::UniqueViolated => write!(f, "UniqueViolated"),
			Self::NotNoneViolated => write!(f, "NotNoneViolated"),
			Self::DuplicateAttrName(x) => write!(f, "DuplicateAttrName: `{x}`"),
			Self::DuplicateClassName(x) => write!(f, "DuplicateClassName: `{x}`"),
		}
	}
}

impl Error for MetastoreError {
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum PipestoreError {
	/// Database error
	DbError(Box<dyn Error>),
}

impl Display for PipestoreError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
		}
	}
}

impl Error for PipestoreError {
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
