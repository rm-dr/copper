use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{
	api::RouterState,
	maindb::auth::{errors::CreateUserError, UserId},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct SetPasswordRequest {
	/// The user to modify
	user: UserId,

	/// The new password to set
	new_password: String,

	/// The setting user's password.
	/// we re-authenticate here, just in case.
	my_password: String,
}

/// Create a new user
#[utoipa::path(
	post,
	path = "/user/set_password",
	responses(
		(status = 200, description = "Successfully set user password"),
		(status = 400, description = "Could not change password"),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized", body = String)
	)
)]
pub(super) async fn set_user_password(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<SetPasswordRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => {
			match state
				.main_db
				.auth
				.test_password(&u.name, &payload.my_password)
				.await
			{
				Ok(Some(_)) => {}
				Ok(None) => {
					info!(
						message = "User tried to change password, but messed up their own.",
						source_user = ?u,
						?payload
					);
					return (StatusCode::UNAUTHORIZED, "Unauthorized: bad password")
						.into_response();
				}
				Err(e) => {
					error!(
						message = "Could not check password",
						cookies = ?jar,
						error = ?e
					);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						"Could not check password",
					)
						.into_response();
				}
			}

			if !u.group.permissions.edit_users_sub.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			let target_user = match state.main_db.auth.get_user(payload.user).await {
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
				Ok(is_parent) => is_parent,
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
			if !(u.group.id == target_user.group.id || is_parent) {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}
	}

	match state
		.main_db
		.auth
		.set_user_password(payload.user, &payload.new_password)
		.await
	{
		Ok(()) => {
			return StatusCode::OK.into_response();
		}

		Err(CreateUserError::BadPassword) => {
			return (StatusCode::BAD_REQUEST, Json("New password isn't valid")).into_response();
		}

		Err(CreateUserError::DbError(e)) => {
			error!(
				message = "Could not change password",
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Could not change password"),
			)
				.into_response();
		}

		Err(CreateUserError::BadName(_))
		| Err(CreateUserError::AlreadyExists)
		| Err(CreateUserError::BadGroup) => {
			unreachable!()
		}
	};
}
