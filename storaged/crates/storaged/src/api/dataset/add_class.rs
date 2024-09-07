use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_database::api::{client::DatabaseClient, errors::itemclass::AddItemclassError};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewItemclassRequest {
	name: String,
}

/// Create a new itemclass
#[utoipa::path(
	post,
	path = "/{dataset_id}/class",
	responses(
		(status = 200, description = "Itemclass created successfully"),
		(status = 400, description = "Bad request", body = String),
		(status = 404, description = "Dataset does not exist"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_itemclass<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<u32>,
	Json(payload): Json<NewItemclassRequest>,
) -> Response {
	let res = state
		.client
		.add_itemclass(dataset_id.into(), &payload.name)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),
		Err(AddItemclassError::NoSuchDataset) => StatusCode::NOT_FOUND.into_response(),
		Err(AddItemclassError::DbError(e)) => {
			error!(
				message = "Database error while making new itemclass",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
