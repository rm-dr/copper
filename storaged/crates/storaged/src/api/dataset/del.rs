use axum::{
	extract::{Path, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
};
use copper_database::api::{errors::dataset::DeleteDatasetError, DatabaseClient};
use tracing::error;

use crate::api::RouterState;

/// Delete a dataset
#[utoipa::path(
	delete,
	path = "/{dataset_id}",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Dataset deleted successfully"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn del_dataset<Client: DatabaseClient>(
	_headers: HeaderMap,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<u32>,
) -> Response {
	let res = state.client.del_dataset(dataset_id.into()).await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),
		Err(DeleteDatasetError::DbError(error)) => {
			error!(
				message = "Database error while deleting dataset",
				dataset_id,
				?error,
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
