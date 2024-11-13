use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;

mod attr;

use attr::*;

#[derive(OpenApi)]
#[openapi(tags(), paths(get_attr), components(schemas()))]
pub(super) struct ItemApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new().route("/:item_idx/attr/:attr_idx", get(get_attr))
}
