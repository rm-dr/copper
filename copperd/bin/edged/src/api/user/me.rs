use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storage::database::base::client::StorageDatabaseClient;

/// Get logged in user info
#[utoipa::path(
	get,
	path = "/me",
	responses(
		(status = 200, description = "Logged in user info", body = UserInfo),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn get_me<Client: DatabaseClient, StorageClient: StorageDatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, StorageClient>>,
) -> Response {
	match state.auth.auth_or_logout(&state, &jar).await {
		Err(response) => response,
		Ok(user) => (StatusCode::OK, Json(user)).into_response(),
	}
}
