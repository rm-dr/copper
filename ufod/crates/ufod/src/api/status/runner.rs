use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_pipeline::{
	api::PipelineNodeState,
	labels::{PipelineName, PipelineNodeID},
};
use utoipa::ToSchema;

use crate::RouterState;

/// This server's pipeline runner status
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct RunnerStatus {
	/// How many jobs are queued to run?
	pub queued_jobs: usize,

	/// How many jobs have been finished?
	pub finished_jobs: usize,

	/// How many jobs have failed
	pub failed_jobs: usize,

	/// What jobs are running right now?
	pub running_jobs: Vec<RunningJobStatus>,
}

/// A running pipeline job's status
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct RunningJobStatus {
	/// This job's id
	pub job_id: u128,

	/// The pipeline this job is running
	#[schema(value_type = String)]
	pub pipeline: PipelineName,

	/// The status of each node in this pipline
	pub node_status: Vec<RunningNodeStatus>,

	/// This pipeline's input, converted to a pretty string.
	/// Context-dependent.
	pub input_exemplar: String,
}

/// The state of a node in a running pipeline
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct RunningNodeStatus {
	/// This node's name
	#[schema(value_type = String)]
	pub name: PipelineNodeID,

	/// This node's state
	pub state: RunningNodeState,
}

/// A running node's state
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum RunningNodeState {
	/// This node is not done, and is not running.
	Pending {
		/// Why this node is pending
		message: String,
	},

	/// This node is running
	Running,

	/// This node is done
	Done,
}

/// Get information about this server's pipeline runner
#[utoipa::path(
	get,
	path = "/runner",
	responses(
		(status = 200, description = "Pipeline runner status", body = RunnerStatus),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn get_runner_status(
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

	let running_jobs: Vec<RunningJobStatus> = runner
		.iter_active_jobs()
		.map(|(job_id, job)| {
			let p = job.get_pipeline();
			RunningJobStatus {
				job_id: *job_id,
				pipeline: p.get_name().clone(),
				input_exemplar: format!("{:?}", job.get_input().first().unwrap()),
				node_status: p
					.iter_node_ids()
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
			failed_jobs: runner.get_failed_jobs().len(),
			running_jobs,
		}),
	)
		.into_response();
}
