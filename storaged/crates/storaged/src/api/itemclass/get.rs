use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_database::api::{client::DatabaseClient, errors::itemclass::GetItemclassError};
use tracing::error;

/// Get itemclass info
#[utoipa::path(
	get,
	path = "/{itemclass_id}",
	params(
		("itemclass_id", description = "Itemclass id"),
	),
	responses(
		(status = 200, description = "Itemclass info", body = ItemclassInfo),
		(status = 404, description = "Itemclass not found"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn get_itemclass<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(itemclass_id): Path<u32>,
) -> Response {
	return match state.client.get_itemclass(itemclass_id.into()).await {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),
		Err(GetItemclassError::NotFound) => StatusCode::NOT_FOUND.into_response(),
		Err(GetItemclassError::DbError(e)) => {
			error!(
				message = "Database error while getting itemclass",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
