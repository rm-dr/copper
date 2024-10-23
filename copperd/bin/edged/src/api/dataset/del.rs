use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::{
	client::ItemdbClient,
	errors::dataset::{DeleteDatasetError, GetDatasetError},
};
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
	)
)]
pub(super) async fn del_dataset<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(dataset_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	match state.itemdb_client.get_dataset(dataset_id.into()).await {
		Ok(x) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}

		Err(GetDatasetError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetDatasetError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	let res = state.itemdb_client.del_dataset(dataset_id.into()).await;

	return match res {
		Ok(()) => StatusCode::OK.into_response(),

		Err(DeleteDatasetError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
