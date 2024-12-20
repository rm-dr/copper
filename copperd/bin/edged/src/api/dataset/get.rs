use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::errors::dataset::GetDatasetError;
use sqlx::Acquire;
use tracing::error;

/// Get dataset info
#[utoipa::path(
	get,
	path = "/{dataset_id}",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Dataset info", body = DatasetInfo),
		(status = 401, description = "Unauthorized"),
		(status = 404, description = "Dataset not found"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn get_dataset<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let mut conn = match state.itemdb_client.new_connection().await {
		Ok(x) => x,
		Err(error) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	let mut trans = match conn.begin().await {
		Ok(y) => y,
		Err(error) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	return match state
		.itemdb_client
		.get_dataset(&mut trans, dataset_id.into())
		.await
	{
		Ok(x) => {
			if x.owner != user.id {
				return (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response();
			}
			(StatusCode::OK, Json(x)).into_response()
		}

		Err(GetDatasetError::NotFound) => {
			return (StatusCode::NOT_FOUND, Json("Dataset not found")).into_response()
		}

		Err(GetDatasetError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};
}
