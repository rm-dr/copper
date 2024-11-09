use axum::{
	extract::{ConnectInfo, State},
	http::{header::SET_COOKIE, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
	Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use copper_itemdb::client::base::client::ItemdbClient;
use serde::Deserialize;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{auth::AUTH_COOKIE_NAME, database::base::client::DatabaseClient, CopperConnectInfo};

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
pub(super) async fn try_login<Client: DatabaseClient, Itemdb: ItemdbClient>(
	ConnectInfo(connect_info): ConnectInfo<CopperConnectInfo>,
	State(state): State<RouterState<Client, Itemdb>>,
	Json(payload): Json<LoginRequest>,
) -> Response {
	info!(
		message = "Received login request",
		client_ip = ?connect_info.addr.ip(),
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
				client_ip = ?connect_info.addr.ip(),
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

		Ok(None) => {
			info!(
				message = "Login failed",
				client_ip = ?connect_info.addr.ip(),
				payload = ?payload,
			);

			return (StatusCode::BAD_REQUEST, Json("Login failed")).into_response();
		}

		Err(e) => {
			error!(
				message = "Could not auth user",
				client_ip = ?connect_info.addr.ip(),
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Could not auth user"),
			)
				.into_response();
		}
	};
}
