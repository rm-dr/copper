use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
};
use copper_database::api::{client::DatabaseClient, errors::class::DeleteClassError};
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
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<u32>,
) -> Response {
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
