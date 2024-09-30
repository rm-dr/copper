use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_storaged::client::StoragedRequestError;
use tracing::error;

use crate::api::RouterState;

/// Delete a attribute
#[utoipa::path(
	delete,
	path = "/{attribute_id}",
	params(
		("attribute_id", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "Attribute deleted successfully"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn del_attribute<Client: DatabaseClient>(
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

	let class = match state.storaged_client.get_class(attr.class).await {
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
		.del_attribute(attribute_id.into())
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

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
