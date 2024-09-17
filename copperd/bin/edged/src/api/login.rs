use axum::{
	extract::State,
	http::{header::SET_COOKIE, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
	Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{auth::AUTH_COOKIE_NAME, database::base::client::DatabaseClient};

use super::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct LoginRequest {
	email: String,
	password: String,
}

/// Try to log in
#[utoipa::path(
	post,
	path = "/login",
	responses(
		(status = 200, description = "Successfully logged in", body=String),
		(status = 400, description = "Could not log in"),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn try_login<Client: DatabaseClient>(
	State(state): State<RouterState<Client>>,
	Json(payload): Json<LoginRequest>,
) -> Response {
	info!(
		message = "Received login request",
		payload = ?payload
	);

	match state
		.auth
		.try_login(&state, &payload.email, &payload.password)
		.await
	{
		Ok(Some(x)) => {
			info!(
				message = "Successfully logged in",
				auth_info = ?x.user,
				payload = ?payload
			);

			let token = x.token.to_string();

			let cookie = Cookie::build((AUTH_COOKIE_NAME, token))
				.path("/")
				.secure(true)
				.http_only(true)
				.same_site(SameSite::None)
				.expires(x.expires);

			return (
				AppendHeaders([(SET_COOKIE, cookie.to_string())]),
				Json("Login successful, cookie set"),
			)
				.into_response();
		}

		Ok(None) => return StatusCode::BAD_REQUEST.into_response(),

		Err(e) => {
			error!(
				message = "Could not auth user",
				request_payload = ?payload,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not auth user").into_response();
		}
	};
}