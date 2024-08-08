use std::{sync::Arc, time::Instant};

use axum::{
	extract::Path,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::{info, warn};
use ufo_util::mime::MimeType;
use utoipa::ToSchema;

use crate::uploader::{UploadJobFile, Uploader};

/// Parameters to start a new file
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadStartInfo {
	/// This file's extension, used to determine its mime type
	#[schema(value_type = String)]
	pub file_extension: SmartString<LazyCompact>,
}

/// A freshly-started upload file's parameters
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadNewFileResult {
	/// This file's name
	#[schema(value_type = String)]
	pub file_name: SmartString<LazyCompact>,
}

/// Start a file inside an upload job and return its handle
#[utoipa::path(
	post,
	path = "/{job_id}/newfile",
	params(
		("job_id" = String, description = "Upload job id")
	),
	responses(
		(status = 200, description = "New file info", body = UploadNewFileResult),
		(status = 404, description = "This job id doesn't exist"),
		(
			status = 500,
			description = "Internal error, check server logs. Should not appear during normal operation.",
			body = String,
			example = json!("error text")
		)
	),
)]
pub(super) async fn start_file(
	uploader: Arc<Uploader>,
	Path(job_id): Path<SmartString<LazyCompact>>,
	Json(start_info): Json<UploadStartInfo>,
) -> Response {
	let mut jobs = uploader.jobs.lock().await;

	// Try to find the given job
	let job = match jobs.iter_mut().find(|us| us.id == job_id) {
		Some(x) => x,
		None => {
			warn!(
				message = "Tried to start a file in a job that doesn't exist",
				bad_job_id = ?job_id,
			);

			return (
				StatusCode::NOT_FOUND,
				format!("upload job {job_id} does not exist"),
			)
				.into_response();
		}
	};
	job.last_activity = Instant::now();

	let file_mime = MimeType::from_extension(&start_info.file_extension).unwrap_or(MimeType::Blob);

	// Make a new handle for this file
	let file_name = loop {
		let id = Uploader::generate_id();
		if job.files.iter().all(|us| us.name != id) {
			break format!("{}{}", id, file_mime.extension());
		}
	};

	job.files.push(UploadJobFile {
		name: file_name.clone().into(),
		file_type: file_mime.clone(),
		is_done: false,
	});

	info!(
		message = "Created a new upload file",
		job = ?job.id,
		file_name= ?file_name,
		file_type = ?file_mime
	);

	return (
		StatusCode::OK,
		Json(UploadNewFileResult {
			file_name: file_name.into(),
		}),
	)
		.into_response();
}
