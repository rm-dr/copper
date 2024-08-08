use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::{debug, error};
use utoipa::ToSchema;

use crate::{
	api::RouterState,
	helpers::maindb::{dataset::DatasetType, errors::CreateDatasetError},
};

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

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum NewDatasetError {
	BadName(String),
	AlreadyExists,
}

/// Create a new dataset
#[utoipa::path(
	post,
	path = "/new",
		responses(
		(status = 200, description = "Dataset created successfully"),
		(status = 400, description = "Could not create dataset", body = NewDatasetError),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn new_dataset(
	State(state): State<RouterState>,
	Json(new_params): Json<NewDataset>,
) -> Response {
	debug!(message = "Making new dataset", new_params=?new_params);

	match new_params.params {
		NewDatasetParams::LocalDataset { .. } => {
			let res = state
				.main_db
				.new_dataset(&new_params.name, DatasetType::Local);

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
