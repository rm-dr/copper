use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_database::api::{client::DatabaseClient, errors::attribute::RenameAttributeError};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct RenameAttributeRequest {
	pub new_name: String,
}

/// Rename a attribute
#[utoipa::path(
	patch,
	path = "/{attribute_id}",
	params(
		("attribute_id", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "Attribute renamed successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn rename_attribute<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(attribute_id): Path<u32>,
	Json(payload): Json<RenameAttributeRequest>,
) -> Response {
	let res = state
		.client
		.rename_attribute(attribute_id.into(), &payload.new_name)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(RenameAttributeError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(RenameAttributeError::DbError(e)) => {
			error!(
				message = "Database error while renaming attribute",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
