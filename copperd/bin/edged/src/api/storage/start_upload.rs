use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_util::MimeType;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{api::RouterState, uploader::errors::NewUploadError};
use crate::{database::base::client::DatabaseClient, uploader::UploadJobId};

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct StartUploadRequest {
	#[schema(value_type = String)]
	mime: MimeType,
}

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct StartUploadResponse {
	#[schema(value_type = String)]
	job_id: UploadJobId,
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
pub(super) async fn start_upload<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
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
				job_id,
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
