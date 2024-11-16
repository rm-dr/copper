use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::{
	client::errors::{
		attribute::AddAttributeError, class::GetClassError, dataset::GetDatasetError,
	},
	AttrDataStub, AttributeOptions,
};
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewAttributeRequest {
	name: String,
	data_type: AttrDataStub,
	options: AttributeOptions,
}

/// Create a new attribute
#[utoipa::path(
	post,
	path = "/{class_id}/attribute",
	params(
		("class_id", description = "Class id"),
	),
	responses(
		(status = 200, description = "Attribute created successfully", body = i64),
		(status = 400, description = "Bad request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 404, description = "Dataset does not exist"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn add_attribute<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(class_id): Path<i64>,
	Json(payload): Json<NewAttributeRequest>,
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
		.add_attribute(
			&mut trans,
			class_id.into(),
			&payload.name,
			payload.data_type,
			payload.options,
		)
		.await;

	return match res {
		Ok(x) => {
			match trans.commit().await {
				Ok(()) => {}
				Err(error) => {
					error!(message = "Error while committing transaction", ?error);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						Json("Internal server error"),
					)
						.into_response();
				}
			};

			(StatusCode::OK, Json(x)).into_response()
		}

		Err(AddAttributeError::NoSuchClass) => {
			return (StatusCode::NOT_FOUND, Json("Class not found")).into_response();
		}

		Err(AddAttributeError::UniqueViolation) => {
			return (
				StatusCode::CONFLICT,
				Json("An attribute with this name already exists"),
			)
				.into_response();
		}

		Err(AddAttributeError::NameError(msg)) => {
			return (StatusCode::BAD_REQUEST, Json(format!("{}", msg))).into_response();
		}

		Err(AddAttributeError::CreatedNotNullWhenItemsExist) => {
			return (
				StatusCode::BAD_REQUEST,
				Json("Cannot create `not null` attribute, this class has items"),
			)
				.into_response();
		}

		Err(AddAttributeError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};
}
