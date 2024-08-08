use std::{error::Error, fmt::Display};

use copper_util::names::NameError;

#[derive(Debug)]
pub enum DeleteGroupError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to delete the root group
	CantDeleteRootGroup,
}

impl Display for DeleteGroupError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::CantDeleteRootGroup => write!(f, "Can't delete root group"),
		}
	}
}

impl Error for DeleteGroupError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CreateUserError {
	/// Database error
	DbError(Box<dyn Error>),

	/// We tried to create a user with an invalid name.
	/// The name error is included.
	BadName(NameError),

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
			Self::BadName(_) => write!(f, "Bad user name"),
			Self::AlreadyExists => write!(f, "User already exists"),
			Self::BadGroup => write!(f, "Invalid group"),
			Self::BadPassword => write!(f, "Tried to make a user with a weak password"),
		}
	}
}

impl Error for CreateUserError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::BadName(x) => Some(x),
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
	BadName(NameError),

	/// A group with this name already exists
	AlreadyExists,

	/// Tried to create a group with a bad parent
	BadParent,
}

impl Display for CreateGroupError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
			Self::BadName(_) => write!(f, "Bad group name"),
			Self::AlreadyExists => write!(f, "Group already exists"),
			Self::BadParent => write!(f, "Bad group parent"),
		}
	}
}

impl Error for CreateGroupError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::BadName(x) => Some(x),
			_ => None,
		}
	}
}
