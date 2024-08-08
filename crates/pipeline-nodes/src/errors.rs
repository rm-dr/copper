//! Errors we may encounter when running a pipeline

use std::{error::Error, fmt::Display};

use ufo_audiofile::flac::errors::FlacError;

/// An error we encountered while running a pipeline
#[derive(Debug)]
pub enum PipelineError {
	/// A generic i/o error.
	IoError(std::io::Error),

	/// We could not understand a flac file TODO: refactor
	FlacError(FlacError),

	FileSystemError(Box<dyn Error>),
	UnsupportedDataType,
}

impl Error for PipelineError {}
unsafe impl Send for PipelineError {}
unsafe impl Sync for PipelineError {}
impl Display for PipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IoError(e) => write!(f, "PipelineIoError: {e}"),
			Self::FlacError(e) => write!(f, "PipelineFlacError: {e}"),
			Self::FileSystemError(e) => write!(f, "PipelineFsError: {e}"),
			Self::UnsupportedDataType => write!(f, "Unsupported Item data type"),
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
