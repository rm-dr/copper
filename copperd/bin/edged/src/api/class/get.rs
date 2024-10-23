use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storage::database::base::{
	client::StorageDatabaseClient,
	errors::{class::GetClassError, dataset::GetDatasetError},
};
use tracing::error;

/// Get class info
#[utoipa::path(
	get,
	path = "/{class_id}",
	params(
		("class_id", description = "Class id"),
	),
	responses(
		(status = 200, description = "Class info", body = ClassInfo),
		(status = 401, description = "Unauthorized"),
		(status = 404, description = "Class not found"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn get_class<Client: DatabaseClient, StorageClient: StorageDatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, StorageClient>>,
	Path(class_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let class = match state.storage_db_client.get_class(class_id.into()).await {
		Ok(x) => x,

		Err(GetClassError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in storage db client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	match state.storage_db_client.get_dataset(class.dataset).await {
		Ok(x) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			return (StatusCode::OK, Json(class)).into_response();
		}

		Err(GetDatasetError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetDatasetError::DbError(error)) => {
			error!(message = "Error in storage db client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
