use crate::database::base::{client::DatabaseClient, errors::dataset::RenameDatasetError};
use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct RenameDatasetRequest {
	pub new_name: String,
}

/// Rename a dataset
#[utoipa::path(
	patch,
	path = "/{dataset_id}",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Dataset renamed successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn rename_dataset<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<u32>,
	Json(payload): Json<RenameDatasetRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let res = state
		.client
		.rename_dataset(dataset_id.into(), &payload.new_name)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(RenameDatasetError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a dataset with this name already exists"),
		)
			.into_response(),

		Err(RenameDatasetError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(RenameDatasetError::DbError(e)) => {
			error!(
				message = "Database error while renaming dataset",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
