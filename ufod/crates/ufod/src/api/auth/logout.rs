use crate::{api::RouterState, helpers::maindb::auth::AUTH_COOKIE_NAME};
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
pub(super) async fn logout(jar: CookieJar, State(state): State<RouterState>) -> Response {
	match state.main_db.auth.terminate_session(&jar).await {
		Some(x) => {
			info!(
				message = "Successfully logged out",
				auth_info = ?x.user,
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
