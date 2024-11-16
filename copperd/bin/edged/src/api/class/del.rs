use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::errors::{
	class::{DeleteClassError, GetClassError},
	dataset::GetDatasetError,
};
use sqlx::Acquire;
use tracing::error;

use crate::api::RouterState;

/// Delete a class
#[utoipa::path(
	delete,
	path = "/{class_id}",
	params(
		("class_id", description = "class id"),
	),
	responses(
		(status = 200, description = "Class deleted successfully"),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn del_class<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<i64>,
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

	let class = match state
		.itemdb_client
		.get_class(&mut trans, class_id.into())
		.await
	{
		Ok(x) => x,

		Err(GetClassError::NotFound) => {
			return (StatusCode::NOT_FOUND, Json("Class not found")).into_response()
		}

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	match state
		.itemdb_client
		.get_dataset(&mut trans, class.dataset)
		.await
	{
		Ok(x) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response();
			}
		}

		// In theory unreachable, but possible with unlucky timing
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
		.del_class(&mut trans, class_id.into())
		.await;

	return match res {
		Ok(()) => match trans.commit().await {
			Ok(()) => StatusCode::OK.into_response(),
			Err(error) => {
				error!(message = "Error while committing transaction", ?error);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					Json("Internal server error"),
				)
					.into_response();
			}
		},

		Err(DeleteClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};
}
