use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{routing::get, Router};
use utoipa::OpenApi;

mod server;

use server::*;

#[derive(OpenApi)]
#[openapi(tags(), paths(get_server_status), components(schemas(ServerStatus,)))]
pub(super) struct StatusApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new().route("/", get(get_server_status))
}
