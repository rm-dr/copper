use crate::database::base::{client::DatabaseClient, errors::user::DeleteUserError};
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tracing::error;

use crate::api::RouterState;

/// Delete a User
#[utoipa::path(
	delete,
	path = "/{user_id}",
	params(
		("user_id", description = "User id"),
	),
	responses(
		(status = 200, description = "User deleted successfully"),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn del_user<Client: DatabaseClient>(
	// OriginalUri(uri): OriginalUri,
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(user_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	// Users can only delete themselves.
	if user.id != user_id.into() {
		return StatusCode::UNAUTHORIZED.into_response();
	}

	let res = state.db_client.del_user(user_id.into()).await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),
		Err(DeleteUserError::DbError(error)) => {
			error!(
				message = "Database error while deleting User",
				user_id,
				?error,
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
