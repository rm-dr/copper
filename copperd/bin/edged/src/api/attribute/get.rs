use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storaged::client::{GenericRequestError, StoragedRequestError};
use tracing::error;

/// Get attribute info
#[utoipa::path(
	get,
	path = "/{attribute_id}",
	params(
		("attribute_id", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "Attribute info", body = AttributeInfo),
		(status = 404, description = "Attribute not found"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn get_attribute<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(attribute_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let attr = match state
		.storaged_client
		.get_attribute(attribute_id.into())
		.await
	{
		Ok(Ok(None)) => return StatusCode::NOT_FOUND.into_response(),

		Ok(Ok(Some(x))) => x,

		Ok(Err(GenericRequestError { code, message })) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}

		Err(StoragedRequestError::RequestError { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	let class = match state.storaged_client.get_class(attr.class).await {
		Ok(Ok(None)) => return StatusCode::NOT_FOUND.into_response(),

		Ok(Ok(Some(x))) => x,

		Ok(Err(GenericRequestError { code, message })) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}

		Err(StoragedRequestError::RequestError { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	match state.storaged_client.get_dataset(class.dataset).await {
		Ok(Ok(None)) => return StatusCode::NOT_FOUND.into_response(),

		Ok(Ok(Some(x))) => {
			// We can only modify our own datasets
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
			return (StatusCode::OK, Json(attr)).into_response();
		}

		Ok(Err(GenericRequestError { code, message })) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}

		Err(StoragedRequestError::RequestError { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
