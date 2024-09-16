use axum::{
	extract::State,
	http::{header::SET_COOKIE, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::{
	cookie::{Cookie, Expiration, SameSite},
	CookieJar,
};
use time::OffsetDateTime;
use tracing::info;

use super::RouterState;
use crate::{auth::AUTH_COOKIE_NAME, database::base::client::DatabaseClient};

/// Terminate this session
#[utoipa::path(
	post,
	path = "/logout",
	responses(
		(status = 200, description = "Successfully terminated session"),
		(status = 400, description = "Could not log out"),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn logout<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
) -> Response {
	info!(message = "Received logout request", cookies = ?jar);

	match state.auth.terminate_session(&jar).await {
		Some(token) => {
			info!(
				message = "Successfully logged out",
				auth_info = ?token.user,
			);

			let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
				.path("/")
				.secure(true)
				.http_only(true)
				.same_site(SameSite::None)
				.expires(Expiration::from(OffsetDateTime::UNIX_EPOCH));

			return (
				AppendHeaders([(SET_COOKIE, cookie.to_string())]),
				"Logout successful",
			)
				.into_response();
		}

		None => {
			return (StatusCode::OK, "No session to log out of").into_response();
		}
	};
}
