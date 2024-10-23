use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{routing::get, Router};
use copper_jobqueue::info::{QueuedJobInfoList, QueuedJobInfoShort, QueuedJobState};
use copper_storage::database::base::client::StorageDatabaseClient;
use utoipa::OpenApi;

mod list;

use list::*;

#[allow(non_camel_case_types)]
#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_jobs),
	components(schemas(QueuedJobInfoList, QueuedJobInfoShort, QueuedJobState))
)]
pub(super) struct JobApi;

pub(super) fn router<
	Client: DatabaseClient + 'static,
	StorageClient: StorageDatabaseClient + 'static,
>() -> Router<RouterState<Client, StorageClient>> {
	Router::new().route("/list", get(list_jobs))
}
