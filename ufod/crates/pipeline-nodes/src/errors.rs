//! Errors we may encounter when running a pipeline

use std::{error::Error, fmt::Display};

use ufo_audiofile::flac::errors::FlacError;
use ufo_db_metastore::errors::MetastoreError;

/// An error we encountered while running a pipeline
#[derive(Debug)]
pub enum PipelineError {
	/// A generic i/o error.
	IoError(std::io::Error),

	/// We could not understand a flac file TODO: refactor
	FlacError(FlacError),

	/// A database operation returned an error
	DatabaseError(MetastoreError),

	FileSystemError(Box<dyn Error>),

	/// We were given an unsupported data type.
	/// Contains a helpful message.
	UnsupportedDataType(String),
}

impl Error for PipelineError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			Self::IoError(e) => e,
			Self::FlacError(e) => e,
			Self::DatabaseError(e) => e,
			Self::FileSystemError(e) => e.as_ref(),
			_ => return None,
		})
	}
}
unsafe impl Send for PipelineError {}
unsafe impl Sync for PipelineError {}
impl Display for PipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(_) => write!(f, "Pipeline i/o error"),
			Self::FlacError(_) => write!(f, "Pipeline flac error"),
			Self::DatabaseError(_) => write!(f, "Pipeline database error"),
			Self::FileSystemError(_) => write!(f, "Pipeline filesystem error"),
			Self::UnsupportedDataType(m) => write!(f, "Unsupported Item data type: {m}"),
		}
	}
}

impl From<std::io::Error> for PipelineError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<FlacError> for PipelineError {
	fn from(value: FlacError) -> Self {
		Self::FlacError(value)
	}
}

impl From<MetastoreError> for PipelineError {
	fn from(value: MetastoreError) -> Self {
		Self::DatabaseError(value)
	}
}
