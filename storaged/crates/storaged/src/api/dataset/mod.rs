use crate::RouterState;
use axum::{
	routing::{delete, get, patch, post},
	Router,
};
use copper_database::api::client::DatabaseClient;
use utoipa::OpenApi;

mod add;
mod add_class;
mod del;
mod get;
mod rename;

use add::*;
use add_class::*;
use del::*;
use get::*;
use rename::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(add_dataset, rename_dataset, del_dataset, get_dataset),
	components(schemas(RenameDatasetRequest, NewDatasetRequest))
)]
pub(super) struct DatasetApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new()
		.route("/", post(add_dataset))
		.route("/:dataset_id", get(get_dataset))
		.route("/:dataset_id", delete(del_dataset))
		.route("/:dataset_id", patch(rename_dataset))
		//
		.route("/:dataset_id/class", post(add_class))
}
