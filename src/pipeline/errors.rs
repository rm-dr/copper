use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum PipelineError {
	/// We tried to run a pipeline that hasn't been checked
	PipelineUnchecked,

	/// We tried to run a pipeline that has failed a check
	PipelineCheckFailed,

	// Need to be cleaned up
	FileSystemError(Box<dyn Error>),
	UnsupportedDataType,
}

// TODO: clean up
impl Error for PipelineError {}
unsafe impl Send for PipelineError {}
unsafe impl Sync for PipelineError {}
impl Display for PipelineError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::PipelineUnchecked => write!(f, "Tried to run an unchecked pipeline"),
			Self::PipelineCheckFailed => write!(f, "Tried to run a pipeline that failed `check()`"),
			Self::FileSystemError(e) => write!(f, "Fs error: {e}"),
			Self::UnsupportedDataType => write!(f, "Unsupported Item data type"),
		}
	}
}
