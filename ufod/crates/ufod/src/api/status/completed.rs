use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_pipeline::labels::PipelineName;
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
		(status = 200, description = "Completed jobs", body=Vec<CompletedJobStatus>),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn get_runner_completed(
	jar: CookieJar,
	State(state): State<RouterState>,
) -> Response {
	match state.main_db.auth.check_cookies(&jar).await {
		Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
		Ok(Some(_)) => {}
		Err(e) => {
			error!(
				message = "Could not check auth cookies",
				cookies = ?jar,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not check auth cookies"),
			)
				.into_response();
		}
	}

	let runner = state.runner.lock().await;

	let completed_jobs: Vec<CompletedJobStatus> = runner
		.get_completed_jobs()
		.iter()
		.map(|c| CompletedJobStatus {
			job_id: c.job_id,
			pipeline: c.pipeline.clone(),
			input_exemplar: format!("{:?}", c.input.first().unwrap()),
		})
		.collect();

	return (StatusCode::OK, Json(completed_jobs)).into_response();
}
