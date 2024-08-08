use crate::RouterState;
use axum::{
	routing::{delete, get, post},
	Router,
};
use utoipa::OpenApi;

mod add;
mod del;
mod list;

use add::*;
use del::*;
use list::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(add_dataset, list_datasets, del_dataset),
	components(schemas(
		NewDatasetRequest,
		NewDatasetParams,
		DatasetInfoShort,
		DeleteDatasetRequest,
	))
)]
pub(super) struct DatasetApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/list", get(list_datasets))
		.route("/add", post(add_dataset))
		.route("/del", delete(del_dataset))
}
