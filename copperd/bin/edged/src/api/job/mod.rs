use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{routing::get, Router};
use copper_itemdb::client::base::client::ItemdbClient;
use copper_jobqueue::info::{QueuedJobInfoList, QueuedJobInfoShort, QueuedJobState};
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

pub(super) fn router<Client: DatabaseClient + 'static, Itemdb: ItemdbClient + 'static>(
) -> Router<RouterState<Client, Itemdb>> {
	Router::new().route("/list", get(list_jobs))
}
