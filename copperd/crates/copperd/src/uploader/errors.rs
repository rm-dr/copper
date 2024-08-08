use std::{error::Error, fmt::Display};

use axum::{http::StatusCode, response::IntoResponse};
use smartstring::{LazyCompact, SmartString};

#[derive(Debug)]
pub enum JobBindError {
	/// We tried to bind a job that doesn't exist
	NoSuchJob,

	/// We tried to bind a job that has already been bound
	AlreadyBound,
}

#[derive(Debug)]
pub enum UploadNewFileError {
	/// We tried to make a new file in a job that doesn't exist
	BadUploadJob,
}
impl Display for UploadNewFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BadUploadJob => {
				write!(f, "Tried to start an upload in a job that doesn't exist")
			}
		}
	}
}
impl Error for UploadNewFileError {}

impl IntoResponse for UploadNewFileError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::BadUploadJob => {
				(StatusCode::NOT_FOUND, "This upload job does not exist").into_response()
			}
		}
	}
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum UploadFinishFileError {
	/// We tried to finish a file in a job that doesn't exist
	BadUploadJob,

	/// We tried to finish a file that doesn't exist
	BadFileID,

	/// We tried to finish a finished file
	AlreadyFinished,

	/// Tried to finish a file with missing fragments
	MissingFragments {
		job_id: SmartString<LazyCompact>,
		file_id: SmartString<LazyCompact>,
		expected_fragments: u32,
		missing_fragment: u32,
	},

	/// I/O error while finishing file
	IoError(std::io::Error),

	/// Final hash doesn't match
	HashDoesntMatch {
		actual: SmartString<LazyCompact>,
		expected: SmartString<LazyCompact>,
	},
}

impl From<std::io::Error> for UploadFinishFileError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl Display for UploadFinishFileError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BadUploadJob => {
				write!(f, "Tried to finish an upload in a job that doesn't exist")
			}
			Self::BadFileID => {
				write!(f, "Tried to finish an upload file that doesn't exist")
			}
			Self::MissingFragments { .. } => {
				write!(f, "Tried to finish a file with missing fragments")
			}
			Self::AlreadyFinished => {
				write!(f, "Tried to finish a finished file")
			}
			Self::IoError(_) => write!(f, "I/O error while finishing file"),
			Self::HashDoesntMatch { .. } => {
				write!(f, "Final upload hash doesn't match expected hash")
			}
		}
	}
}

impl Error for UploadFinishFileError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::IoError(x) => Some(x),
			_ => None,
		}
	}
}

impl IntoResponse for UploadFinishFileError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::BadUploadJob => {
				(StatusCode::NOT_FOUND, "This upload job does not exist").into_response()
			}
			Self::BadFileID => {
				(StatusCode::NOT_FOUND, "This file id does not exist").into_response()
			}
			Self::MissingFragments { .. } => (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Could not finish file with missing fragments",
			)
				.into_response(),
			Self::AlreadyFinished => (
				StatusCode::BAD_REQUEST,
				"This file has already been finished",
			)
				.into_response(),
			Self::IoError { .. } => (
				StatusCode::INTERNAL_SERVER_ERROR,
				"I/O error while finishing file",
			)
				.into_response(),
			Self::HashDoesntMatch { .. } => (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Final file hash doesn't match",
			)
				.into_response(),
		}
	}
}

#[derive(Debug)]
pub enum UploadFragmentError {
	/// We tried to push a fragment to a file in a job that doesn't exist
	BadUploadJob,

	/// We tried to push a fragment to a file that doesn't exist
	BadFileID,

	/// We tried to push a fragment to a file that has been finished
	AlreadyFinished,

	/// I/O error while processing fragment
	IoError(std::io::Error),
}

impl From<std::io::Error> for UploadFragmentError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl Display for UploadFragmentError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BadUploadJob => {
				write!(
					f,
					"Tried to push a fragment to a file in a job that doesn't exist"
				)
			}
			Self::BadFileID => {
				write!(f, "Tried to push a fragment to a file that doesn't exist")
			}
			Self::AlreadyFinished => {
				write!(f, "Tried to push a fragment to a finished file")
			}
			Self::IoError(_) => write!(f, "I/O error while processing fragment"),
		}
	}
}

impl Error for UploadFragmentError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::IoError(x) => Some(x),
			_ => None,
		}
	}
}

impl IntoResponse for UploadFragmentError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::BadUploadJob => {
				(StatusCode::NOT_FOUND, "This upload job does not exist").into_response()
			}
			Self::BadFileID => {
				(StatusCode::NOT_FOUND, "This file id does not exist").into_response()
			}
			Self::AlreadyFinished => (
				StatusCode::BAD_REQUEST,
				"Cannot push a fragment to a finished file",
			)
				.into_response(),
			Self::IoError { .. } => (
				StatusCode::INTERNAL_SERVER_ERROR,
				"I/O error while consuming fragment",
			)
				.into_response(),
		}
	}
}
