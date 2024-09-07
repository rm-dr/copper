use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_database::api::{
	client::{AttributeOptions, DatabaseClient},
	data::AttrDataStub,
	errors::attribute::AddAttributeError,
};
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
	responses(
		(status = 200, description = "Attribute created successfully"),
		(status = 400, description = "Bad request", body = String),
		(status = 404, description = "Dataset does not exist"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_attribute<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<u32>,
	Json(payload): Json<NewAttributeRequest>,
) -> Response {
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
		Ok(_) => StatusCode::OK.into_response(),
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
