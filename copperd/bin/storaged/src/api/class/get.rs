use crate::database::base::{client::DatabaseClient, errors::class::GetClassError};
use crate::RouterState;
use axum::extract::OriginalUri;
use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use tracing::error;

/// Get class info
#[utoipa::path(
	get,
	path = "/{class_id}",
	params(
		("class_id", description = "Class id"),
	),
	responses(
		(status = 200, description = "Class info", body = ClassInfo),
		(status = 404, description = "Class not found"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn get_class<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<i64>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	return match state.client.get_class(class_id.into()).await {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),
		Err(GetClassError::NotFound) => StatusCode::NOT_FOUND.into_response(),
		Err(GetClassError::DbError(e)) => {
			error!(
				message = "Database error while getting class",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
