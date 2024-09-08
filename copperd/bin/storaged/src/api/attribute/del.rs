use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
};
use storaged_database::api::{client::DatabaseClient, errors::attribute::DeleteAttributeError};
use tracing::error;

use crate::api::RouterState;

/// Delete a attribute
#[utoipa::path(
	delete,
	path = "/{attribute_id}",
	params(
		("attribute_id", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "Attribute deleted successfully"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn del_attribute<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(attribute_id): Path<u32>,
) -> Response {
	let res = state.client.del_attribute(attribute_id.into()).await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),
		Err(DeleteAttributeError::DbError(error)) => {
			error!(
				message = "Database error while deleting attribute",
				attribute_id,
				?error,
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
