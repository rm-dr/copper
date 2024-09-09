use crate::database::base::{client::DatabaseClient, errors::dataset::GetDatasetError};
use crate::RouterState;
use axum::extract::OriginalUri;
use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use tracing::error;

/// Get dataset info
#[utoipa::path(
	get,
	path = "/{dataset_id}",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Dataset info", body = DatasetInfo),
		(status = 404, description = "Dataset not found"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn get_dataset<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<u32>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	return match state.client.get_dataset(dataset_id.into()).await {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),
		Err(GetDatasetError::NotFound) => StatusCode::NOT_FOUND.into_response(),
		Err(GetDatasetError::DbError(e)) => {
			error!(
				message = "Database error while getting dataset",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
