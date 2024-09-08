use crate::database::base::{client::DatabaseClient, errors::class::GetClassError};
use crate::RouterState;
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
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<u32>,
) -> Response {
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
