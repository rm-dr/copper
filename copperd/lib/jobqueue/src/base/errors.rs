//! Errors we can encounter when operating on datasets

use std::{error::Error, fmt::Display};

/// An error we can encounter when creating a job
#[derive(Debug)]
pub enum AddJobError {
	/// Database error
	DbError(sqlx::Error),

	/// A job with this id already exists
	AlreadyExists,
}

impl Display for AddJobError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::AlreadyExists => write!(f, "a job with this id already exists"),
		}
	}
}

impl Error for AddJobError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for AddJobError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when getting a job by id
#[derive(Debug)]
pub enum GetJobShortError {
	/// Database error
	DbError(sqlx::Error),

	/// A job with this id doesn't exist
	NotFound,
}

impl Display for GetJobShortError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NotFound => write!(f, "a job with this id doesn't exist"),
		}
	}
}

impl Error for GetJobShortError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for GetJobShortError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// An error we can encounter when listing a user's jobs
#[derive(Debug)]
pub enum GetUserJobsError {
	/// Database error
	DbError(sqlx::Error),
}

impl Display for GetUserJobsError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for GetUserJobsError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x),
		}
	}
}

impl From<sqlx::Error> for GetUserJobsError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}
