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
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum CreateUserError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to create a user with an invalid name.
	/// The name error is included.
	BadName(String),

	/// We tried to create a user with a weak password
	BadPassword,

	/// A user with this name already exists
	AlreadyExists,

	/// Tried to create a user with an invalid group
	BadGroup,
}

impl Display for CreateUserError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::BadName(message) => write!(f, "Bad user name: {message}"),
			Self::AlreadyExists => write!(f, "User already exists"),
			Self::BadGroup => write!(f, "Invalid group"),
			Self::BadPassword => write!(f, "Tried to make a user with a weak password"),
		}
	}
}

impl Error for CreateUserError {
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum CreateGroupError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to create a group with an invalid name.
	/// The name error is included.
	BadName(String),

	/// A group with this name already exists
	AlreadyExists,

	/// Tried to create a group with a bad parent
	BadParent,
}

impl Display for CreateGroupError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::BadName(message) => write!(f, "Bad group name: {message}"),
			Self::AlreadyExists => write!(f, "Group already exists"),
			Self::BadParent => write!(f, "Bad group parent"),
		}
	}
}

impl Error for CreateGroupError {
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}
