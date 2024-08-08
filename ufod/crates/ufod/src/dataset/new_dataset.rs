use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::debug;
use utoipa::ToSchema;

/// New dataset creation parameters
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewDataset {
	/// The name of this dataset
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	pub params: NewDatasetParams,
}

/// Types of datasets we support, with options
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum NewDatasetParams {
	/// A dataset stored locally
	LocalDataset {
		metadata_type: LocalDatasetMetadataType,
	},
}

/// How a local dataset should store its metadata
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) enum LocalDatasetMetadataType {
	Sqlite,
}

/// Create a new dataset
#[utoipa::path(
	post,
	path = "/new",
		responses(
		(status = 200, description = "Dataset created successfully"),
		(status = 500, description = "A dataset with this name already exists"),
	),
)]
pub(super) async fn new_dataset(Json(new_params): Json<NewDataset>) -> Response {
	debug!(message = "Making new dataset", new_params=?new_params);
	return StatusCode::OK.into_response();
}
