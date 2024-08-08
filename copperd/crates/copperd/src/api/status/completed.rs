use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_pipeline::labels::PipelineName;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::RouterState;

/// Completed pipeline job status
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct CompletedJobStatus {
	/// This job's id
	pub job_id: u128,

	/// The pipeline this job ran
	#[schema(value_type = String)]
	pub pipeline: PipelineName,

	// TODO: redo
	/// A pretty string that identifies this job by its input
	pub input_exemplar: String,
}

/// Get a list of completed pipeline jobs
#[utoipa::path(
	get,
	path = "/runner/completed",
	responses(
		(status = 200, description = "Completed jobs", body = Vec<CompletedJobStatus>),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn get_runner_completed(
	jar: CookieJar,
	State(state): State<RouterState>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	let runner = state.runner.lock().await;

	let completed_jobs: Vec<CompletedJobStatus> = runner
		.get_completed_jobs()
		.iter()
		.map(|c| CompletedJobStatus {
			job_id: c.job_id,
			pipeline: c.pipeline.clone(),
			input_exemplar: format!("{:?}", c.input.first_key_value().unwrap().0),
		})
		.collect();

	return (StatusCode::OK, Json(completed_jobs)).into_response();
}