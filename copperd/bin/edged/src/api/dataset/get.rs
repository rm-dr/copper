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

	return match state.storaged_client.get_dataset(dataset_id.into()).await {
		Ok(Ok(None)) => StatusCode::NOT_FOUND.into_response(),

		Ok(Ok(Some(x))) => {
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
			(StatusCode::OK, Json(x)).into_response()
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
