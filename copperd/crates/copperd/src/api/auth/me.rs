use crate::api::RouterState;
use axum::{
	extract::State,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;

/// Get logged in user's info
#[utoipa::path(
	get,
	path = "/me",
	responses(
		(status = 200, description = "User info", body = UserInfo),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn get_me(jar: CookieJar, State(state): State<RouterState>) -> Response {
	let user_info = match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => u,
	};

	return Json(user_info).into_response();
}
