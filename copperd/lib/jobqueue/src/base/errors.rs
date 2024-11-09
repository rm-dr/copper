//! Errors we can encounter when operating on datasets

use thiserror::Error;

/// An error we can encounter when creating a job
#[derive(Debug, Error)]
pub enum AddJobError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// A job with this id already exists
	#[error("a job with this id already exists")]
	AlreadyExists,
}

/// An error we can encounter when getting a job by id
#[derive(Debug, Error)]
pub enum GetJobShortError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// A job with this id doesn't exist
	#[error("a job with this id doesn't exist")]
	NotFound,
}

/// An error we can encounter when listing a user's jobs
#[derive(Debug, Error)]
pub enum GetUserJobsError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}

/// An error we can encounter when getting a queued job
#[derive(Debug, Error)]
pub enum GetQueuedJobError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),
}

/// An error we can encounter when marking a job as `BuildError`
#[derive(Debug, Error)]
pub enum BuildErrorJobError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// This job is not running
	#[error("job is not running")]
	NotRunning,
}

/// An error we can encounter when marking a job as `Failed`
#[derive(Debug, Error)]
pub enum FailJobError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// This job is not running
	#[error("job is not running")]
	NotRunning,
}

/// An error we can encounter when marking a job as `Success`
#[derive(Debug, Error)]
pub enum SuccessJobError {
	/// Database error
	#[error("database backend error")]
	DbError(#[from] sqlx::Error),

	/// This job is not running
	#[error("job is not running")]
	NotRunning,
}
