use crate::database::base::{client::DatabaseClient, errors::class::AddClassError};
use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewClassRequest {
	name: String,
}

/// Create a new class
#[utoipa::path(
	post,
	path = "/{dataset_id}/class",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Class created successfully", body = u32),
		(status = 400, description = "Bad request", body = String),
		(status = 404, description = "Dataset does not exist"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_class<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<i64>,
	Json(payload): Json<NewClassRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let res = state
		.client
		.add_class(dataset_id.into(), &payload.name)
		.await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(AddClassError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(AddClassError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a class with this name already exists"),
		)
			.into_response(),

		Err(AddClassError::NoSuchDataset) => StatusCode::NOT_FOUND.into_response(),

		Err(AddClassError::DbError(e)) => {
			error!(
				message = "Database error while making new class",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
