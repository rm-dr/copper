use crate::database::base::{client::DatabaseClient, errors::attribute::GetAttributeError};
use crate::RouterState;
use axum::extract::OriginalUri;
use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use tracing::error;

/// Get attribute info
#[utoipa::path(
	get,
	path = "/{attribute_id}",
	params(
		("attribute_id", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "Attribute info", body = AttributeInfo),
		(status = 404, description = "Attribute not found"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn get_attribute<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(attribute_id): Path<i64>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	return match state.client.get_attribute(attribute_id.into()).await {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),
		Err(GetAttributeError::NotFound) => StatusCode::NOT_FOUND.into_response(),
		Err(GetAttributeError::DbError(e)) => {
			error!(
				message = "Database error while getting attribute",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
