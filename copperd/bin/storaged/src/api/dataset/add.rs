use crate::database::base::{client::DatabaseClient, errors::dataset::AddDatasetError};
use axum::{
	extract::{OriginalUri, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_edged::UserId;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewDatasetRequest {
	#[schema(value_type = i64)]
	owner: UserId,

	name: String,
}

/// Create a new dataset
#[utoipa::path(
	post,
	path = "",
	responses(
		(status = 200, description = "Dataset created successfully", body = i64),
		(status = 400, description = "Bad request", body = String),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_dataset<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Json(payload): Json<NewDatasetRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let res = state.client.add_dataset(&payload.name, payload.owner).await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(AddDatasetError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(AddDatasetError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a dataset with this name already exists"),
		)
			.into_response(),

		Err(AddDatasetError::DbError(e)) => {
			error!(
				message = "Database error while making new dataset",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
