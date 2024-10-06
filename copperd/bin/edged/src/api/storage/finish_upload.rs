use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tracing::error;

use crate::database::base::client::DatabaseClient;
use crate::{api::RouterState, uploader::errors::UploadFinishError};

/// Rename a attribute
#[utoipa::path(
	post,
	path = "/upload/{upload_id}/finish",
	params(
		("upload_id", description = "Upload id"),
	),
	responses(
		(status = 200, description = "Upload finished successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 401, description = "Unauthorized", body = String),
		(status = 404, description = "Upload not found", body = String),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn finish_upload<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(job_id): Path<String>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	return match state.uploader.finish_job(user.id, &job_id).await {
		Ok(()) => StatusCode::OK.into_response(),

		Err(UploadFinishError::NotMyUpload) | Err(UploadFinishError::BadUpload) => {
			return StatusCode::NOT_FOUND.into_response();
		}

		Err(UploadFinishError::S3Error(error)) => {
			error!(message = "S3 error while finishing job", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
