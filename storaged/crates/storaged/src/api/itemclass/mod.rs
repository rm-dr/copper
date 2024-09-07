use crate::RouterState;
use axum::{
	routing::{delete, get, patch},
	Router,
};
use copper_database::api::client::DatabaseClient;
use utoipa::OpenApi;

mod del;
mod get;
mod rename;

use del::*;
use get::*;
use rename::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(rename_itemclass, del_itemclass, get_itemclass),
	components(schemas(RenameItemclassRequest))
)]
pub(super) struct ItemclassApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new()
		.route("/:itemclass_id", get(get_itemclass))
		.route("/:itemclass_id", delete(del_itemclass))
		.route("/:itemclass_id", patch(rename_itemclass))
}
