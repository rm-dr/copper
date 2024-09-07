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
	paths(rename_class, del_class, get_class),
	components(schemas(RenameClassRequest))
)]
pub(super) struct ClassApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new()
		.route("/:class_id", get(get_class))
		.route("/:class_id", delete(del_class))
		.route("/:class_id", patch(rename_class))
}
