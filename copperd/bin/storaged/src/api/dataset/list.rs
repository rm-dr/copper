use crate::database::base::client::DatabaseClient;
use crate::database::base::errors::dataset::ListDatasetsError;
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
	path = "/owned_by/{user_id}",
	params(
		("user_id", description = "User id"),
	),
	responses(
		(status = 200, description = "This user's datasets", body = Vec<DatasetInfo>),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn list_datasets<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(user_id): Path<i64>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	return match state.client.list_datasets(user_id.into()).await {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),
		Err(ListDatasetsError::DbError(e)) => {
			error!(
				message = "Database error while listing datasets",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
