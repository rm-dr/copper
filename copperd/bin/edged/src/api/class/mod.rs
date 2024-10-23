use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	routing::{delete, get, patch, post},
	Router,
};
use copper_storage::database::base::client::StorageDatabaseClient;
use utoipa::OpenApi;

mod add_attribute;
mod del;
mod get;
mod rename;

use add_attribute::*;
use del::*;
use get::*;
use rename::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(rename_class, del_class, get_class, add_attribute),
	components(schemas(RenameClassRequest, NewAttributeRequest))
)]
pub(super) struct ClassApi;

pub(super) fn router<
	Client: DatabaseClient + 'static,
	StorageClient: StorageDatabaseClient + 'static,
>() -> Router<RouterState<Client, StorageClient>> {
	Router::new()
		.route("/:class_id", get(get_class))
		.route("/:class_id", delete(del_class))
		.route("/:class_id", patch(rename_class))
		//
		.route("/:class_id/attribute", post(add_attribute))
}
