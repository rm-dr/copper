use axum::{
	extract::{OriginalUri, Path, Query, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_pipelined::structs::{JobCounts, JobInfo, JobInfoList};
use copper_storaged::UserId;
use serde::Deserialize;
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::{pipeline::runner::JobState, RouterState};

const MAX_PAGE_COUNT: usize = 200;

#[derive(Debug, Deserialize, IntoParams)]
pub(super) struct PaginateParams {
	skip: usize,
	count: usize,
}

/// Start a pipeline job
#[utoipa::path(
	get,
	path = "/list/{user_id}",
	params(PaginateParams),
	responses(
		(status = 200, description = "This user's jobs, ordered by age", body = JobInfoList),
		(status = 401, description = "Unauthorized"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn list_jobs(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState>,
	Path(user_id): Path<String>,
	Query(mut paginate): Query<PaginateParams>,
) -> Response {
	let user_id: UserId = match user_id.parse::<i64>() {
		Ok(x) => x.into(),
		Err(_) => return StatusCode::BAD_REQUEST.into_response(),
	};

	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	if paginate.count > MAX_PAGE_COUNT {
		warn!(
			message = "Page size is too big, limiting to maximum",
			requested = paginate.count,
			maximum = MAX_PAGE_COUNT
		);
		paginate.count = MAX_PAGE_COUNT;
	}

	let runner = state.runner.lock().await;

	let mut jobs: Vec<JobInfo> = Vec::new();
	let jbu = match runner.jobs_by_user(user_id) {
		Some(x) => x,

		None => {
			return (
				StatusCode::OK,
				Json(JobInfoList {
					counts: JobCounts {
						total_jobs: 0,
						queued_jobs: 0,
						running_jobs: 0,
						successful_jobs: 0,
						failed_jobs: 0,
						build_errors: 0,
					},

					skip: paginate.skip,
					jobs: Vec::new(),
				}),
			)
				.into_response();
		}
	};

	// Iterate in reverse so newest jobs come first
	let mut counts = JobCounts {
		total_jobs: jbu.len(),
		queued_jobs: 0,
		running_jobs: 0,
		successful_jobs: 0,
		failed_jobs: 0,
		build_errors: 0,
	};

	for job_id in jbu.iter().rev().skip(paginate.skip) {
		let job = match runner.get_job(job_id) {
			Some(x) => x,
			None => {
				// Probably a race condition.
				info!(message = "Job id was not found by `get_job`", ?job_id);
				continue;
			}
		};

		if job.owner != user_id {
			continue;
		}

		if jobs.len() < paginate.count {
			jobs.push({
				JobInfo {
					id: job.id.to_string(),
					owner: job.owner,
					state: (&job.state).into(),
					added_at: job.added_at,
					started_at: job.started_at,
					finished_at: job.finished_at,
				}
			});
		}

		match &job.state {
			JobState::Queued { .. } => counts.queued_jobs += 1,
			JobState::Running => counts.running_jobs += 1,
			JobState::Failed => counts.failed_jobs += 1,
			JobState::Success => counts.successful_jobs += 1,
			JobState::BuildError(_) => counts.build_errors += 1,
		}
	}

	return (
		StatusCode::OK,
		Json(JobInfoList {
			counts,
			skip: paginate.skip,
			jobs,
		}),
	)
		.into_response();
}
