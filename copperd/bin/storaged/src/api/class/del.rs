use crate::database::base::{client::DatabaseClient, errors::class::DeleteClassError};
use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
};
use tracing::error;

use crate::api::RouterState;

/// Delete a class
#[utoipa::path(
	delete,
	path = "/{class_id}",
	params(
		("class_id", description = "class id"),
	),
	responses(
		(status = 200, description = "Class deleted successfully"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn del_class<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<i64>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};
	let res = state.client.del_class(class_id.into()).await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),
		Err(DeleteClassError::DbError(error)) => {
			error!(
				message = "Database error while deleting class",
				class_id,
				?error,
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
