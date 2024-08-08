use crate::RouterState;
use axum::{routing::post, Router};
use utoipa::OpenApi;

mod new_dataset;

use new_dataset::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(new_dataset),
	components(schemas(NewDataset, NewDatasetParams, LocalDatasetMetadataType))
)]
pub(super) struct DatasetApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		//.route("/", get(get_server_status))
		//.route("/runner", get(get_runner_status))
		.route("/new", post(new_dataset))
}
