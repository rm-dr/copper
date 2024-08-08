use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct LoginRequest {
	username: String,
	password: String,
}

/// Try to log in
#[utoipa::path(
	post,
	path = "/login",
	responses(
		(status = 200, description = "Successfully logged in"),
		(status = 400, description = "Could not log in"),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn try_login(
	State(state): State<RouterState>,
	Json(payload): Json<LoginRequest>,
) -> Response {
	info!(
		message = "Received login request",
		payload = ?payload
	);

	match state
		.main_db
		.try_auth_user(&payload.username, &payload.password)
		.await
	{
		Ok(Some(x)) => {
			info!(
				message = "Successfully logged in",
				auth_info = ?x.user,
				payload = ?payload
			);
			return x.token.to_string().into_response();
		}

		Ok(None) => return StatusCode::BAD_REQUEST.into_response(),

		Err(e) => {
			error!(
				message = "Could not auth user",
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not auth user"),
			)
				.into_response();
		}
	};
}
