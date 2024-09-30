use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storaged::{client::StoragedRequestError, AttrDataStub, AttributeOptions};
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

	let class = match state.storaged_client.get_class(class_id.into()).await {
		Ok(None) => return StatusCode::NOT_FOUND.into_response(),

		Ok(Some(x)) => x,

		Err(StoragedRequestError::Other { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(StoragedRequestError::GenericHttp { code, message }) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}
	};

	match state.storaged_client.get_dataset(class.dataset).await {
		Ok(None) => return StatusCode::NOT_FOUND.into_response(),

		Ok(Some(x)) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}

		Err(StoragedRequestError::Other { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(StoragedRequestError::GenericHttp { code, message }) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}
	};

	let res = state
		.storaged_client
		.add_attribute(
			class_id.into(),
			&payload.name,
			payload.data_type,
			payload.options,
		)
		.await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(StoragedRequestError::Other { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(StoragedRequestError::GenericHttp { code, message }) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}
	};
}
