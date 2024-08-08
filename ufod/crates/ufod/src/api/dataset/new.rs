use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use utoipa::ToSchema;

use crate::{
	api::RouterState,
	helpers::maindb::{dataset::DatasetType, errors::CreateDatasetError},
};

/// Types of datasets we support, with options
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum NewDatasetParams {
	/// A dataset stored locally
	LocalDataset,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum NewDatasetError {
	BadName(String),
	AlreadyExists,
}

/// Create a new dataset
#[utoipa::path(
	post,
	path = "/{dataset_name}",
	params(
		("dataset_name" = String, description = "Dataset name")
	),
	responses(
		(status = 200, description = "Dataset created successfully"),
		(status = 400, description = "Could not create dataset", body = NewDatasetError),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn new_dataset(
	State(state): State<RouterState>,
	Path(dataset_name): Path<String>,
	Json(new_params): Json<NewDatasetParams>,
) -> Response {
	debug!(message = "Making new dataset", new_params=?new_params);

	match new_params {
		NewDatasetParams::LocalDataset => {
			let res = state.main_db.new_dataset(&dataset_name, DatasetType::Local);

			match res {
				Ok(_) => {}
				Err(CreateDatasetError::BadName(message)) => {
					return (
						StatusCode::BAD_REQUEST,
						Json(NewDatasetError::BadName(message)),
					)
						.into_response()
				}
				Err(CreateDatasetError::AlreadyExists(_)) => {
					return (
						StatusCode::BAD_REQUEST,
						Json(NewDatasetError::AlreadyExists),
					)
						.into_response();
				}
				Err(CreateDatasetError::DbError(e)) => {
					error!(
						message = "Database error while making new dataset",
						error = ?e
					);
					return StatusCode::INTERNAL_SERVER_ERROR.into_response();
				}
			};
		}
	}

	return StatusCode::OK.into_response();
}
