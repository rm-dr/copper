use crate::RouterState;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::get,
	Json, Router,
};
use ufo_api::status::{
	CompletedJobStatus, RunnerStatus, RunningJobStatus, RunningNodeState, RunningNodeStatus,
	ServerStatus,
};
use ufo_pipeline::api::PipelineNodeState;

pub fn router() -> Router<RouterState> {
	Router::new()
		.route("/", get(get_server_status))
		.route("/runner", get(get_runner_status))
		.route("/runner/completed", get(get_runner_completed))
}

async fn get_server_status(State(state): State<RouterState>) -> Response {
	return (
		StatusCode::OK,
		Json(ServerStatus {
			version: env!("CARGO_PKG_VERSION").into(),
			request_body_limit: state.config.request_body_limit,
		}),
	)
		.into_response();
}

async fn get_runner_status(State(state): State<RouterState>) -> Response {
	let runner = state.runner.lock().await;

	let running_jobs: Vec<RunningJobStatus> = runner
		.iter_active_jobs()
		.map(|(job_id, job)| {
			let p = job.get_pipeline();
			RunningJobStatus {
				job_id: *job_id,
				pipeline: p.get_name().clone(),
				input_exemplar: format!("{:?}", job.get_input().first().unwrap()),
				node_status: p
					.iter_node_labels()
					.map(|l| RunningNodeStatus {
						name: l.clone(),
						state: match job.get_node_status(l).unwrap() {
							(true, _) => RunningNodeState::Running,
							(false, PipelineNodeState::Done) => RunningNodeState::Done,
							(false, PipelineNodeState::Pending(m)) => {
								RunningNodeState::Pending { message: m.into() }
							}
						},
					})
					.collect(),
			}
		})
		.collect();

	return (
		StatusCode::OK,
		Json(RunnerStatus {
			queued_jobs: runner.get_queued_jobs().len(),
			finished_jobs: runner.get_completed_jobs().len(),
			running_jobs,
		}),
	)
		.into_response();
}

async fn get_runner_completed(State(state): State<RouterState>) -> Response {
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
