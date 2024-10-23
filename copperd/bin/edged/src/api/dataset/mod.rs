use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	routing::{delete, get, patch, post},
	Router,
};
use copper_storage::database::base::client::StorageDatabaseClient;
use utoipa::OpenApi;

mod add;
mod add_class;
mod del;
mod get;
mod list;
mod rename;

use add::*;
use add_class::*;
use del::*;
use get::*;
use list::*;
use rename::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(
		add_dataset,
		rename_dataset,
		del_dataset,
		get_dataset,
		add_class,
		list_datasets
	),
	components(schemas(RenameDatasetRequest, NewDatasetRequest, NewClassRequest))
)]
pub(super) struct DatasetApi;

pub(super) fn router<
	Client: DatabaseClient + 'static,
	StorageClient: StorageDatabaseClient + 'static,
>() -> Router<RouterState<Client, StorageClient>> {
	Router::new()
		.route("/", post(add_dataset))
		.route("/:dataset_id", get(get_dataset))
		.route("/:dataset_id", delete(del_dataset))
		.route("/:dataset_id", patch(rename_dataset))
		//
		.route("/list", get(list_datasets))
		.route("/:dataset_id/class", post(add_class))
}
