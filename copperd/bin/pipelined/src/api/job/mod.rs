use axum::{routing::get, Router};
use utoipa::OpenApi;

mod get;
use get::*;

use super::RouterState;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(get_job),
	components(schemas(JobResponse, JobResponseState))
)]
pub(super) struct JobApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new().route("/:job_id", get(get_job))
}
