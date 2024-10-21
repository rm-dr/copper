use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_storaged::client::{GenericRequestError, StoragedRequestError};
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

	let class = match state.storaged_client.get_class(class_id.into()).await {
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

	let res = state.storaged_client.del_class(class_id.into()).await;

	return match res {
		Ok(Ok(())) => StatusCode::OK.into_response(),

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
