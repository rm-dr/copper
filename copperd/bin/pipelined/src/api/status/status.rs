use axum::{
	extract::{OriginalUri, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::RouterState;

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct StatusResponse {
	queued_jobs: usize,
	running_jobs: usize,
	max_running_jobs: usize,
}

/// Start a pipeline job
#[utoipa::path(
	get,
	path = "",
	responses(
		(status = 200, description = "Pipelined status", body = StatusResponse),
		(status = 401, description = "Unauthorized"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn get_status(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let runner = state.runner.lock().await;

	let status = StatusResponse {
		queued_jobs: runner.queued_jobs().len(),
		running_jobs: runner.running_jobs().len(),
		max_running_jobs: state.config.pipelined_max_running_jobs,
	};

	return (StatusCode::OK, Json(status)).into_response();
}
