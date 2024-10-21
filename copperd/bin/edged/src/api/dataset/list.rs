use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::State,
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
	path = "/list",
		responses(
		(status = 200, description = "This user's datasets", body = Vec<DatasetInfo>),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn list_datasets<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	return match state.storaged_client.list_datasets(user.id).await {
		Ok(Ok(x)) => (StatusCode::OK, Json(x)).into_response(),

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
