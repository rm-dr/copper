use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::Serialize;
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::{pipeline::runner::JobState, RouterState};

#[derive(Debug, Serialize, ToSchema)]
pub(super) struct JobResponse {
	id: String,

	state: JobResponseState,
	added_at: OffsetDateTime,
	started_at: Option<OffsetDateTime>,
	finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(super) enum JobResponseState {
	Queued,
	Running,
	Success,
	Failed,
	BuildError { message: String },
}

/// Start a pipeline job
#[utoipa::path(
	get,
	path = "/{job_id}",
	responses(
		(status = 200, description = "Job status", body = JobResponse),
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
			let status = JobResponse {
				id: job.id.to_string(),

				state: match &job.state {
					JobState::Queued { .. } => JobResponseState::Queued,
					JobState::Running => JobResponseState::Running,
					JobState::Failed => JobResponseState::Failed,
					JobState::Success => JobResponseState::Success,
					JobState::BuildError(err) => JobResponseState::BuildError {
						message: format!("{err}"),
					},
				},

				added_at: job.added_at,
				started_at: job.started_at,
				finished_at: job.finished_at,
			};
			(StatusCode::OK, Json(status)).into_response()
		}
		None => StatusCode::NOT_FOUND.into_response(),
	};
}
