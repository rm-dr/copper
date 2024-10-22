use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{routing::get, Router};
use copper_jobqueue::info::{QueuedJobInfoList, QueuedJobInfoShort, QueuedJobStateShort};
use utoipa::OpenApi;

mod list;

use list::*;

#[allow(non_camel_case_types)]
#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_jobs),
	components(schemas(QueuedJobInfoList, QueuedJobInfoShort, QueuedJobStateShort))
)]
pub(super) struct JobApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new().route("/list", get(list_jobs))
}
