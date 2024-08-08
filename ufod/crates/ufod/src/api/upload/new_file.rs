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
use ufo_util::mime::MimeType;
use utoipa::ToSchema;

use crate::api::RouterState;

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
		)
	),
)]
pub(super) async fn start_file(
	jar: CookieJar,
	State(state): State<RouterState>,
	Path(job_id): Path<SmartString<LazyCompact>>,
	Json(start_info): Json<UploadStartInfo>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let t_job_id = job_id.clone();
	let mime = MimeType::from_extension(&start_info.file_extension).unwrap_or(MimeType::Blob);
	match tokio::task::spawn_blocking(move || state.uploader.new_file(&t_job_id, mime)).await {
		Ok(Ok(file_id)) => {
			return (
				StatusCode::OK,
				Json(UploadNewFileResult {
					file_id: file_id.into(),
				}),
			)
				.into_response();
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
