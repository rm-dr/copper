use copper_util::s3client::{S3CreateMultipartUploadError, S3UploadFinishError, S3UploadPartError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NewUploadError {
	/// S3 client error while creating upload
	#[error("S3 client error while processing fragment")]
	S3Error(#[from] S3CreateMultipartUploadError),
}

#[derive(Debug, Error)]
pub enum UploadFragmentError {
	/// We tried to push a fragment to an upload that doesn't exist or isn't pending
	#[error("tried to push a fragment to an upload that doesn't exist")]
	BadUpload,

	/// We tried to push a fragment to an upload we don't own
	#[error("tried to push a fragment to an upload that we don't own")]
	NotMyUpload,

	/// Fragment is too small
	//PartIsTooSmall,

	/// S3 client error while processing fragment
	#[error("S3 client error while processing fragment")]
	S3Error(#[from] S3UploadPartError),
}

#[derive(Debug, Error)]
pub enum UploadFinishError {
	/// We tried to finish an upload that doesn't exist or isn't pending
	#[error("tried to finish an upload that doesn't exist")]
	BadUpload,

	/// We tried to finish an upload we don't own
	#[error("tried to finish an upload that we don't own")]
	NotMyUpload,

	/// S3 client error while finishing upload
	#[error("S3 client error while finishing an upload")]
	S3Error(#[from] S3UploadFinishError),
}

#[derive(Debug, Error)]
pub enum UploadAssignError {
	/// We tried to assign an upload that doesn't exist or isn't done
	#[error("tried to finish an upload that doesn't exist")]
	BadUpload,

	/// We tried to assign an upload we don't own
	#[error("tried to finish an upload that we don't own")]
	NotMyUpload,
}
