use copper_pipelined::helpers::{
	S3CreateMultipartUploadError, S3UploadFinishError, S3UploadPartError,
};
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum NewUploadError {
	/// S3 client error while creating upload
	S3Error(S3CreateMultipartUploadError),
}

impl From<S3CreateMultipartUploadError> for NewUploadError {
	fn from(value: S3CreateMultipartUploadError) -> Self {
		Self::S3Error(value)
	}
}

impl Display for NewUploadError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::S3Error(_) => write!(f, "S3 client error while processing fragment"),
		}
	}
}

impl Error for NewUploadError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::S3Error(x) => Some(x),
		}
	}
}

#[derive(Debug)]
pub enum UploadFragmentError {
	/// We tried to push a fragment to an upload that doesn't exist
	BadUpload,

	/// We tried to push a fragment to an upload we don't own
	NotMyUpload,

	/// S3 client error while processing fragment
	S3Error(S3UploadPartError),
}

impl From<S3UploadPartError> for UploadFragmentError {
	fn from(value: S3UploadPartError) -> Self {
		Self::S3Error(value)
	}
}

impl Display for UploadFragmentError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BadUpload => {
				write!(
					f,
					"Tried to push a fragment to an upload that doesn't exist"
				)
			}
			Self::NotMyUpload => {
				write!(f, "Tried to push a fragment to an upload that we don't own")
			}

			Self::S3Error(_) => write!(f, "S3 client error while processing fragment"),
		}
	}
}

impl Error for UploadFragmentError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::S3Error(x) => Some(x),
			_ => None,
		}
	}
}

#[derive(Debug)]
pub enum UploadFinishError {
	/// We tried to finish an upload that doesn't exist
	BadUpload,

	/// We tried to finish an upload we don't own
	NotMyUpload,

	/// S3 client error while finishing upload
	S3Error(S3UploadFinishError),
}

impl From<S3UploadFinishError> for UploadFinishError {
	fn from(value: S3UploadFinishError) -> Self {
		Self::S3Error(value)
	}
}

impl Display for UploadFinishError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BadUpload => {
				write!(f, "Tried to finish an upload that doesn't exist")
			}
			Self::NotMyUpload => {
				write!(f, "Tried to finish an upload that we don't own")
			}
			Self::S3Error(_) => write!(f, "S3 client error while finishing an upload"),
		}
	}
}

impl Error for UploadFinishError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::S3Error(x) => Some(x),
			_ => None,
		}
	}
}
