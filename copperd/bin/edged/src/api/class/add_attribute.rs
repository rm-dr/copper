use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::{
	client::base::{
		client::ItemdbClient,
		errors::{attribute::AddAttributeError, class::GetClassError, dataset::GetDatasetError},
	},
	AttrDataStub, AttributeOptions,
};
use serde::{Deserialize, Serialize};
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
pub(super) async fn add_attribute<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(class_id): Path<i64>,
	Json(payload): Json<NewAttributeRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let class = match state.itemdb_client.get_class(class_id.into()).await {
		Ok(x) => x,

		Err(GetClassError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	match state.itemdb_client.get_dataset(class.dataset).await {
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

	let res = state
		.itemdb_client
		.add_attribute(
			class_id.into(),
			&payload.name,
			payload.data_type,
			payload.options,
		)
		.await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(AddAttributeError::NoSuchClass) => {
			return StatusCode::NOT_FOUND.into_response();
		}

		Err(AddAttributeError::UniqueViolation) => {
			return StatusCode::CONFLICT.into_response();
		}

		Err(AddAttributeError::NameError(msg)) => {
			return (StatusCode::BAD_REQUEST, Json(format!("{}", msg))).into_response();
		}

		Err(AddAttributeError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
