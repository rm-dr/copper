use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::{
	client::ItemdbClient,
	errors::{class::AddClassError, dataset::GetDatasetError},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewClassRequest {
	name: String,
}

/// Create a new class
#[utoipa::path(
	post,
	path = "/{dataset_id}/class",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Class created successfully", body = i64),
		(status = 400, description = "Bad request", body = String),
		(status = 404, description = "Dataset does not exist"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn add_class<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(dataset_id): Path<i64>,
	Json(payload): Json<NewClassRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	match state.itemdb_client.get_dataset(dataset_id.into()).await {
		Ok(x) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response();
			}
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

	let res = state
		.itemdb_client
		.add_class(dataset_id.into(), &payload.name)
		.await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(AddClassError::UniqueViolation) => {
			return (
				StatusCode::CONFLICT,
				Json("An attribute with this name already exists"),
			)
				.into_response();
		}

		Err(AddClassError::NoSuchDataset) => {
			return (StatusCode::NOT_FOUND, Json("Dataset not found")).into_response()
		}

		Err(AddClassError::NameError(msg)) => {
			return (StatusCode::BAD_REQUEST, Json(format!("{}", msg))).into_response();
		}

		Err(AddClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};
}
