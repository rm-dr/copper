use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
};
use copper_database::api::{client::DatabaseClient, errors::itemclass::DeleteItemclassError};
use tracing::error;

use crate::api::RouterState;

/// Delete an itemclass
#[utoipa::path(
	delete,
	path = "/{itemclass_id}",
	params(
		("itemclass_id", description = "Itemclass id"),
	),
	responses(
		(status = 200, description = "Itemclass deleted successfully"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn del_itemclass<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(itemclass_id): Path<u32>,
) -> Response {
	let res = state.client.del_itemclass(itemclass_id.into()).await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),
		Err(DeleteItemclassError::DbError(error)) => {
			error!(
				message = "Database error while deleting itemclass",
				itemclass_id,
				?error,
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
