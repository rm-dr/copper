use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CreateDatasetError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to create a dataset with an invalid name.
	/// The name error is included.
	BadName(String),

	/// A dataset with this name already exists
	AlreadyExists,
}

impl Display for CreateDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::BadName(message) => write!(f, "Bad dataset name: {message}"),
			Self::AlreadyExists => write!(f, "Dataset already exists"),
		}
	}
}

impl Error for CreateDatasetError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum RenameDatasetError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to give a dataset an invalid name.
	/// An error message is included.
	BadName(String),

	/// A dataset with this name already exists
	AlreadyExists,
}

impl Display for RenameDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::BadName(message) => write!(f, "Bad dataset name: {message}"),
			Self::AlreadyExists => write!(f, "Dataset already exists"),
		}
	}
}

impl Error for RenameDatasetError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}
