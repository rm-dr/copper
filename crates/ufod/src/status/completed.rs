use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
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

	/// The error we finished with, if any
	pub error: Option<String>,

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
	),
)]
pub(super) async fn get_runner_completed(State(state): State<RouterState>) -> Response {
	let runner = state.runner.lock().await;

	let completed_jobs: Vec<CompletedJobStatus> = runner
		.get_completed_jobs()
		.iter()
		.map(|c| CompletedJobStatus {
			job_id: c.job_id,
			pipeline: c.pipeline.clone(),
			error: c.error.as_ref().map(|x| x.to_string()),
			input_exemplar: format!("{:?}", c.input.first().unwrap()),
		})
		.collect();

	return (StatusCode::OK, Json(completed_jobs)).into_response();
}
