//! Errors we can encounter when operating on datasets

use copper_util::names::NameError;
use std::{error::Error, fmt::Display};

/// An error we can encounter when creating a pipeline
#[derive(Debug)]
pub enum AddPipelineError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// This user already has a pipeline with this name
	UniqueViolation,

	/// We tried to create a pipeline with an invalid name
	NameError(NameError),
}

impl Display for AddPipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
			Self::UniqueViolation => write!(f, "this user already has a pipeline with this name"),
			Self::NameError(_) => write!(f, "invalid pipeline name"),
		}
	}
}

impl Error for AddPipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

/// An error we can encounter when getting a pipeline
#[derive(Debug)]
pub enum GetPipelineError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for GetPipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for GetPipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}

/// An error we can encounter when updating a user
#[derive(Debug)]
pub enum UpdatePipelineError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),

	/// This user already has a pipeline with this name
	UniqueViolation,

	/// We tried to set an invalid name
	NameError(NameError),
}

impl Display for UpdatePipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UniqueViolation => write!(f, "this user already has a pipeline with this name"),
			Self::DbError(_) => write!(f, "database backend error"),
			Self::NameError(_) => write!(f, "invalid user name"),
		}
	}
}

impl Error for UpdatePipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			Self::NameError(x) => Some(x),
			_ => None,
		}
	}
}

/// An error we can encounter when deleting a user
#[derive(Debug)]
pub enum DeletePipelineError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for DeletePipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "database backend error"),
		}
	}
}

impl Error for DeletePipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
		}
	}
}
