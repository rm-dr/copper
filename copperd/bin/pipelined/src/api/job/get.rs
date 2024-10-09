use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_pipelined::structs::JobInfo;

use crate::RouterState;

/// Start a pipeline job
#[utoipa::path(
	get,
	path = "/{job_id}",
	responses(
		(status = 200, description = "Job status", body = JobInfo),
		(status = 401, description = "Unauthorized"),
		(status = 404, description = "Job not found"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn get_job(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState>,
	Path(job_id): Path<String>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let runner = state.runner.lock().await;

	return match runner.get_job(&job_id) {
		Some(job) => {
			let status = JobInfo {
				id: job.id.to_string(),
				owner: job.owner,

				state: (&job.state).into(),
				added_at: job.added_at,
				started_at: job.started_at,
				finished_at: job.finished_at,
			};
			(StatusCode::OK, Json(status)).into_response()
		}
		None => StatusCode::NOT_FOUND.into_response(),
	};
}
