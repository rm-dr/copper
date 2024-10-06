use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_util::MimeType;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::database::base::client::DatabaseClient;
use crate::{api::RouterState, uploader::errors::NewUploadError};

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct StartUploadRequest {
	#[schema(value_type = String)]
	mime: MimeType,
}

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct StartUploadResponse {
	job_id: String,
	request_body_limit: usize,
}

/// Rename a attribute
#[utoipa::path(
	post,
	path = "/upload",
	responses(
		(status = 200, description = "Upload started successfully", body = StartUploadResponse),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn start_upload<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Json(payload): Json<StartUploadRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	return match state.uploader.new_job(user.id, payload.mime).await {
		Ok(job_id) => (
			StatusCode::OK,
			Json(StartUploadResponse {
				job_id: job_id.into(),
				request_body_limit: state.config.edged_request_body_limit,
			}),
		)
			.into_response(),

		Err(NewUploadError::S3Error(error)) => {
			error!(message = "S3 error while creating upload job", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
