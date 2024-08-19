use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{api::RouterState, maindb::auth::UserId};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct DeluserRequest {
	#[schema(value_type = u32)]
	user: UserId,
}

/// Delete a user
#[utoipa::path(
	delete,
	path = "/user/del",
	responses(
		(status = 200, description = "Successfully deleted user"),
		(status = 400, description = "Could not delete user"),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn del_user(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<DeluserRequest>,
) -> Response {
	let userinfo = match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => {
			if !u.group.permissions.edit_users_sub.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			let target_user = match state.main_db.auth.get_user(payload.user.into()).await {
				Ok(x) => x,
				Err(e) => {
					error!(
						message = "Could not check group parent",
						cookies = ?jar,
						error = ?e
					);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						"Could not check group parent",
					)
						.into_response();
				}
			};

			// We need a special permission if we want to edit users in our group
			if u.group.id == target_user.group.id
				&& !u.group.permissions.edit_users_same.is_allowed()
			{
				return StatusCode::UNAUTHORIZED.into_response();
			}

			// Is the group we want to create a user in a child of this user's group?
			let is_parent = match state
				.main_db
				.auth
				.is_group_parent(u.group.id, target_user.group.id)
				.await
			{
				Ok(x) => x,
				Err(e) => {
					error!(
						message = "Could not check group parent",
						cookies = ?jar,
						error = ?e
					);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						"Could not check group parent",
					)
						.into_response();
				}
			};

			// We can only create users in our group,
			// or in groups that are subgroups of our group.
			if !(is_parent || u.group.id == target_user.group.id) {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			// Return user info, we'll need it later
			u
		}
	};

	if u32::from(payload.user) == u32::from(userinfo.id) {
		return (StatusCode::BAD_REQUEST, "Cannot delete self").into_response();
	}

	match state.main_db.auth.del_user(payload.user.into()).await {
		Ok(()) => {
			return StatusCode::OK.into_response();
		}

		Err(e) => {
			error!(
				message = "Could not delete user",
				request_payload = ?payload,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not delete user").into_response();
		}
	};
}
