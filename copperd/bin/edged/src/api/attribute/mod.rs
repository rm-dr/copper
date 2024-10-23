use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	routing::{delete, get, patch},
	Router,
};
use copper_storage::database::base::client::StorageDatabaseClient;
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

pub(super) fn router<
	Client: DatabaseClient + 'static,
	StorageClient: StorageDatabaseClient + 'static,
>() -> Router<RouterState<Client, StorageClient>> {
	Router::new()
		.route("/:attribute_id", get(get_attribute))
		.route("/:attribute_id", delete(del_attribute))
		.route("/:attribute_id", patch(rename_attribute))
}
