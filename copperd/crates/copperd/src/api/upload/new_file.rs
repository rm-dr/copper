use std::{ffi::OsStr, path::PathBuf};

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::warn;
use copper_util::mime::MimeType;
use utoipa::ToSchema;

use crate::api::RouterState;

/// Parameters to start a new file
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadStartInfo {
	/// This file's name, used to determine its mime type
	#[schema(value_type = String)]
	pub file_name: SmartString<LazyCompact>,
}

/// A freshly-started upload file's parameters
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadNewFileResult {
	/// This file's name
	#[schema(value_type = String)]
	pub file_id: SmartString<LazyCompact>,
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
		),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn start_file(
	jar: CookieJar,
	State(state): State<RouterState>,
	Path(job_id): Path<SmartString<LazyCompact>>,
	Json(start_info): Json<UploadStartInfo>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	let t_job_id = job_id.clone();
	let file_name = PathBuf::from(start_info.file_name.as_str());
	let file_ext = match file_name.extension().unwrap_or(OsStr::new("")).to_str() {
		Some(x) => x,
		None => {
			return (StatusCode::BAD_REQUEST, "File name is not a valid string").into_response();
		}
	};
	let mime = MimeType::from_extension(file_ext).unwrap_or(MimeType::Blob);
	match tokio::task::spawn_blocking(move || state.uploader.new_file(&t_job_id, mime)).await {
		Ok(Ok(file_id)) => {
			return (StatusCode::OK, Json(UploadNewFileResult { file_id })).into_response();
		}

		Err(e) => {
			warn!(
				message = "spawn_blocking exited with error",
				job_id = ?job_id,
				error = ?e
			);

			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"spawn_blocking exited with error",
			)
				.into_response();
		}

		Ok(Err(e)) => {
			warn!(
				message = "Could not create file in upload job",
				upload_job_id = ?job_id,
				error = ?e
			);

			return e.into_response();
		}
	};
}
