use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use ufo_ds_impl::DatasetType;
use utoipa::ToSchema;

use crate::{api::RouterState, helpers::maindb::errors::CreateDatasetError};

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewDatasetRequest {
	name: String,
	params: NewDatasetParams,
}

/// Types of datasets we support, with options
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum NewDatasetParams {
	/// A dataset stored locally
	Local,
}

/// Create a new dataset
#[utoipa::path(
	post,
	path = "/add",
	responses(
		(status = 200, description = "Dataset created successfully"),
		(status = 400, description = "Could not create dataset", body = String),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(super) async fn add_dataset(
	State(state): State<RouterState>,
	Json(payload): Json<NewDatasetRequest>,
) -> Response {
	debug!(message = "Making new dataset", payload = ?payload);

	if payload.name == "" {
		return (
			StatusCode::BAD_REQUEST,
			format!("Dataset name cannot be empty"),
		)
			.into_response();
	} else if payload.name.trim() == "" {
		return (
			StatusCode::BAD_REQUEST,
			format!("Dataset name cannot be whitespace"),
		)
			.into_response();
	}

	match payload.params {
		NewDatasetParams::Local => {
			let res = state
				.main_db
				.new_dataset(&payload.name, DatasetType::Local)
				.await;

			match res {
				Ok(_) => {}
				Err(CreateDatasetError::BadName(message)) => {
					return (StatusCode::BAD_REQUEST, message).into_response()
				}
				Err(CreateDatasetError::AlreadyExists) => {
					return (
						StatusCode::BAD_REQUEST,
						format!("A dataset named `{}` already exists.", payload.name),
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
