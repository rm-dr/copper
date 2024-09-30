use crate::database::base::client::DatabaseClient;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storaged::client::StoragedRequestError;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewDatasetRequest {
	name: String,
}

/// Create a new dataset
#[utoipa::path(
	post,
	path = "",
	responses(
		(status = 200, description = "Dataset created successfully", body = i64),
		(status = 400, description = "Bad request", body = String),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn add_dataset<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Json(payload): Json<NewDatasetRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let res = state
		.storaged_client
		.add_dataset(&payload.name, user.id)
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
