//! Errors we may encounter when running a pipeline

use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum PipelineError {
	FileSystemError(Box<dyn Error>),
	UnsupportedDataType,
}

impl Error for PipelineError {}
unsafe impl Send for PipelineError {}
unsafe impl Sync for PipelineError {}
impl Display for PipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::FileSystemError(e) => write!(f, "Fs error: {e}"),
			Self::UnsupportedDataType => write!(f, "Unsupported Item data type"),
		}
	}
}
