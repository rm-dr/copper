use crate::database::base::{client::DatabaseClient, errors::attribute::DeleteAttributeError};
use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
};
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
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(attribute_id): Path<u32>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

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
