use crate::RouterState;
use axum::{routing::post, Router};
use utoipa::OpenApi;

mod new_dataset;

use new_dataset::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(new_dataset),
	components(schemas(
		NewDataset,
		NewDatasetParams,
		LocalDatasetMetadataType,
		NewDatasetError
	))
)]
pub(super) struct DatasetApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new().route("/new", post(new_dataset))
}
