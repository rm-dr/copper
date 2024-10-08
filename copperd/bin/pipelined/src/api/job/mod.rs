use axum::{routing::get, Router};
use copper_pipelined::structs::{JobCounts, JobInfo, JobInfoList, JobInfoState};
use utoipa::OpenApi;

mod get;
mod list;

use get::*;
use list::*;

use super::RouterState;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(get_job, list_jobs),
	components(schemas(JobInfo, JobInfoState, JobInfoList, JobCounts))
)]
pub(super) struct JobApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/:job_id", get(get_job))
		.route("/list/:user_id", get(list_jobs))
}
