use crate::database::base::{client::DatabaseClient, errors::attribute::AddAttributeError};
use axum::{
	extract::{OriginalUri, Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_storaged::{AttrDataStub, AttributeOptions};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewAttributeRequest {
	name: String,
	data_type: AttrDataStub,
	options: AttributeOptions,
}

/// Create a new attribute
#[utoipa::path(
	post,
	path = "/{class_id}/attribute",
	params(
		("class_id", description = "Class id"),
	),
	responses(
		(status = 200, description = "Attribute created successfully", body = i64),
		(status = 400, description = "Bad request", body = String),
		(status = 404, description = "Dataset does not exist"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_attribute<Client: DatabaseClient>(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<i64>,
	Json(payload): Json<NewAttributeRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let res = state
		.client
		.add_attribute(
			class_id.into(),
			&payload.name,
			payload.data_type,
			payload.options,
		)
		.await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(AddAttributeError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(AddAttributeError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("an attribute with this name already exists"),
		)
			.into_response(),

		Err(AddAttributeError::NoSuchClass) => StatusCode::NOT_FOUND.into_response(),

		Err(AddAttributeError::DbError(e)) => {
			error!(
				message = "Database error while making new attribute",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
