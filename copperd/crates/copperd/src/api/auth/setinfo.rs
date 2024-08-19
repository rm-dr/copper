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

use crate::{
	api::RouterState,
	maindb::auth::{errors::CreateUserError, UserId},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct SetUserInfoRequest {
	/// The user to modify
	#[schema(value_type = u32)]
	user: UserId,

	/// The user's new color
	color: ColorAction,

	/// The user's new email
	email: EmailAction,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "action")]
pub(super) enum EmailAction {
	Unchanged,
	Clear,
	Set { value: String },
}

impl EmailAction {
	fn to_option(&self) -> Option<&str> {
		match self {
			Self::Unchanged => panic!(),
			Self::Clear => None,
			Self::Set { value } => Some(value),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "action")]
pub(super) enum ColorAction {
	Unchanged,
	Set { color: String },
}

/// Create a new user
#[utoipa::path(
	post,
	path = "/user/set_info",
	responses(
		(status = 200, description = "Successfully set user info"),
		(status = 400, description = "Could not change info"),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized", body = String)
	)
)]
pub(super) async fn set_user_info(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<SetUserInfoRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => {
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

	if let ColorAction::Set { color } = &payload.color {
		match state.main_db.auth.set_user_color(payload.user, color).await {
			Ok(()) => {}

			Err(CreateUserError::DbError(e)) => {
				error!(
					message = "Could not set user color",
					request_payload = ?payload,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					Json("Could not set user color"),
				)
					.into_response();
			}

			Err(CreateUserError::BadPassword)
			| Err(CreateUserError::BadName(_))
			| Err(CreateUserError::AlreadyExists)
			| Err(CreateUserError::BadGroup) => {
				unreachable!()
			}
		}
	}

	if !matches!(payload.email, EmailAction::Unchanged) {
		match state
			.main_db
			.auth
			.set_user_email(payload.user, payload.email.to_option())
			.await
		{
			Ok(()) => {}

			Err(CreateUserError::DbError(e)) => {
				error!(
					message = "Could not set user email",
					request_payload = ?payload,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					Json("Could not set user email"),
				)
					.into_response();
			}

			Err(CreateUserError::BadPassword)
			| Err(CreateUserError::BadName(_))
			| Err(CreateUserError::AlreadyExists)
			| Err(CreateUserError::BadGroup) => {
				unreachable!()
			}
		}
	}

	return StatusCode::OK.into_response();
}
