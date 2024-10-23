use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storage::database::base::{
	client::StorageDatabaseClient, errors::dataset::ListDatasetsError,
};
use tracing::error;

/// Get dataset info
#[utoipa::path(
	get,
	path = "/list",
		responses(
		(status = 200, description = "This user's datasets", body = Vec<DatasetInfo>),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn list_datasets<Client: DatabaseClient, StorageClient: StorageDatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, StorageClient>>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	return match state.storage_db_client.list_datasets(user.id).await {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(ListDatasetsError::DbError(error)) => {
			error!(message = "Error in storage db client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
