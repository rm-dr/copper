use crate::database::base::{client::DatabaseClient, errors::class::RenameClassError};
use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct RenameClassRequest {
	pub new_name: String,
}

/// Rename a class
#[utoipa::path(
	patch,
	path = "/{class_id}",
	params(
		("class_id", description = "Class id"),
	),
	responses(
		(status = 200, description = "Class renamed successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn rename_class<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<u32>,
	Json(payload): Json<RenameClassRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let res = state
		.client
		.rename_class(class_id.into(), &payload.new_name)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(RenameClassError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a class with this name already exists"),
		)
			.into_response(),

		Err(RenameClassError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(RenameClassError::DbError(e)) => {
			error!(
				message = "Database error while renaming class",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
