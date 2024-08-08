use std::{error::Error, fmt::Display};

use smartstring::{LazyCompact, SmartString};

#[derive(Debug)]
pub enum CreateDatasetError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to create a dataset with an invalid name.
	/// The name error is included.
	BadName(String),

	/// A dataset with this name already exists
	AlreadyExists(SmartString<LazyCompact>),
}

impl Display for CreateDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::BadName(message) => write!(f, "Bad dataset name: {message}"),
			Self::AlreadyExists(name) => write!(f, "Dataset {name} already exists"),
		}
	}
}

impl Error for CreateDatasetError {
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}
