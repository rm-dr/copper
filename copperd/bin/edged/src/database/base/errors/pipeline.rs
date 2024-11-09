//! Errors we can encounter when operating on datasets

use copper_util::names::NameError;
use thiserror::Error;

/// An error we can encounter when creating a pipeline
#[derive(Debug, Error)]
pub enum AddPipelineError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// This user already has a pipeline with this name
	#[error("this user already has a pipeline with this name")]
	UniqueViolation,

	/// We tried to create a pipeline with an invalid name
	#[error("invalid pipeline name")]
	NameError(#[from] NameError),
}

/// An error we can encounter when getting a pipeline
#[derive(Debug, Error)]
pub enum GetPipelineError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}

/// An error we can encounter when listing a user's pipelines
#[derive(Debug, Error)]
pub enum ListPipelineError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}

/// An error we can encounter when updating a user
#[derive(Debug, Error)]
pub enum UpdatePipelineError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// This user already has a pipeline with this name
	#[error("this user already has a pipeline with this name")]
	UniqueViolation,

	/// We tried to set an invalid name
	#[error("invalid user name")]
	NameError(NameError),
}

/// An error we can encounter when deleting a user
#[derive(Debug, Error)]
pub enum DeletePipelineError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}
