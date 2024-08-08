use std::{error::Error, fmt::Display};

use smartstring::{LazyCompact, SmartString};

#[derive(Debug)]
pub enum DatasetError {
	/// We haven't connected to this database yet
	NotConnected,

	/// SQL error
	SQLX(sqlx::Error),

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

impl Display for DatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotConnected => write!(f, "NotConnected"),
			Self::SQLX(dberr) => write!(f, "DB Error: {}", dberr),
			Self::BadAttrHandle => write!(f, "BadAttrHandle"),
			Self::BadClassHandle => write!(f, "BadClassHandle"),
			Self::TypeMismatch => write!(f, "TypeMismatch"),
			Self::UniqueViolated => write!(f, "UniqueViolated"),
			Self::NotNoneViolated => write!(f, "NotNoneViolated"),
			Self::DuplicateAttrName(_) => write!(f, "DuplicateAttrName"),
			Self::DuplicateClassName(_) => write!(f, "DuplicateClassName"),
		}
	}
}

impl Error for DatasetError {}

impl From<sqlx::Error> for DatasetError {
	fn from(value: sqlx::Error) -> Self {
		Self::SQLX(value)
	}
}
