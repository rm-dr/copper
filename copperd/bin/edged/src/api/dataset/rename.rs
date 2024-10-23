use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storage::database::base::{
	client::StorageDatabaseClient,
	errors::dataset::{GetDatasetError, RenameDatasetError},
};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct RenameDatasetRequest {
	pub new_name: String,
}

/// Rename a dataset
#[utoipa::path(
	patch,
	path = "/{dataset_id}",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Dataset renamed successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn rename_dataset<Client: DatabaseClient, StorageClient: StorageDatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, StorageClient>>,
	Path(dataset_id): Path<i64>,
	Json(payload): Json<RenameDatasetRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	match state.storage_db_client.get_dataset(dataset_id.into()).await {
		Ok(x) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}

		Err(GetDatasetError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetDatasetError::DbError(error)) => {
			error!(message = "Error in storage db client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	let res = state
		.storage_db_client
		.rename_dataset(dataset_id.into(), &payload.new_name)
		.await;

	return match res {
		Ok(()) => StatusCode::OK.into_response(),

		Err(RenameDatasetError::UniqueViolation) => {
			return (
				StatusCode::CONFLICT,
				Json("An attribute with this name already exists"),
			)
				.into_response();
		}

		Err(RenameDatasetError::NameError(msg)) => {
			return (StatusCode::BAD_REQUEST, Json(format!("{}", msg))).into_response();
		}

		Err(RenameDatasetError::DbError(error)) => {
			error!(message = "Error in storage db client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
