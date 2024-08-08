use crate::RouterState;
use axum::{routing::get, Router};
use utoipa::OpenApi;

mod completed;
mod runner;
mod server;

use completed::*;
use runner::*;
use server::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(get_server_status, get_runner_status, get_runner_completed),
	components(schemas(
		ServerStatus,
		RunnerStatus,
		RunningJobStatus,
		RunningNodeStatus,
		RunningNodeState,
		CompletedJobStatus,
	))
)]
pub(super) struct StatusApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/", get(get_server_status))
		.route("/runner", get(get_runner_status))
		.route("/runner/completed", get(get_runner_completed))
}
