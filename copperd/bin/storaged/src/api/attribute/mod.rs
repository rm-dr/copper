use crate::RouterState;
use axum::{
	routing::{delete, get, patch},
	Router,
};
use storaged_database::api::client::DatabaseClient;
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
	paths(rename_attribute, del_attribute, get_attribute),
	components(schemas(RenameAttributeRequest))
)]
pub(super) struct AttributeApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new()
		.route("/:attribute_id", get(get_attribute))
		.route("/:attribute_id", delete(del_attribute))
		.route("/:attribute_id", patch(rename_attribute))
}
