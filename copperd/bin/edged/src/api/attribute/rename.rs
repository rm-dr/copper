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
	errors::{
		attribute::{GetAttributeError, RenameAttributeError},
		class::GetClassError,
		dataset::GetDatasetError,
	},
};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct RenameAttributeRequest {
	pub new_name: String,
}

/// Rename a attribute
#[utoipa::path(
	patch,
	path = "/{attribute_id}",
	params(
		("attribute_id", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "Attribute renamed successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn rename_attribute<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(attribute_id): Path<i64>,
	Json(payload): Json<RenameAttributeRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let attr = match state.itemdb_client.get_attribute(attribute_id.into()).await {
		Ok(x) => x,

		Err(GetAttributeError::NotFound) => {
			return (StatusCode::NOT_FOUND, Json("Attribute not found")).into_response()
		}

		Err(GetAttributeError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	let class = match state.itemdb_client.get_class(attr.class).await {
		Ok(x) => x,

		// In theory unreachable, but possible with unlucky timing
		Err(GetClassError::NotFound) => {
			return (StatusCode::NOT_FOUND, Json("Class not found")).into_response()
		}

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in item db client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	match state.itemdb_client.get_dataset(class.dataset).await {
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
			error!(message = "Error in item db client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	let res = state
		.itemdb_client
		.rename_attribute(attribute_id.into(), &payload.new_name)
		.await;

	return match res {
		Ok(()) => StatusCode::OK.into_response(),

		Err(RenameAttributeError::UniqueViolation) => {
			return (
				StatusCode::CONFLICT,
				Json("An attribute with this name already exists"),
			)
				.into_response();
		}

		Err(RenameAttributeError::NameError(msg)) => {
			return (StatusCode::BAD_REQUEST, Json(format!("{}", msg))).into_response();
		}

		Err(RenameAttributeError::DbError(error)) => {
			error!(message = "Error in item db client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};
}
