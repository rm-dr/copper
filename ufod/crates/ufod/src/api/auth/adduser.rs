use axum::{
	extract::State,
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{
	api::RouterState,
	helpers::maindb::auth::{errors::CreateUserError, GroupId},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct AdduserRequest {
	username: String,
	password: String,
	group: GroupId,
}

/// Create a new user
#[utoipa::path(
	post,
	path = "/user",
	responses(
		(status = 200, description = "Successfully created user"),
		(status = 400, description = "Could not create user"),
		(status = 500, description = "Internal server error", body=String),
		(status = 401, description = "Unauthorized")
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_user(
	headers: HeaderMap,
	State(state): State<RouterState>,
	Json(payload): Json<AdduserRequest>,
) -> Response {
	match state.main_db.auth.check_headers(&headers).await {
		Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
		Ok(Some(u)) => {
			if !u.group.permissions.edit_users_sub.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			// We need a special permission if we want to edit users in our group
			if u.group.id == payload.group && !u.group.permissions.edit_users_same.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			// Is the group we want to create a user in a child of this user's group?
			let is_parent = match state
				.main_db
				.auth
				.is_group_parent(u.group.id, payload.group)
				.await
			{
				Ok(is_parent) => is_parent,
				Err(e) => {
					error!(
						message = "Could not check group parent",
						headers = ?headers,
						error = ?e
					);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						format!("Could not check group parent"),
					)
						.into_response();
				}
			};

			// We can only create users in our group,
			// or in groups that are subgroups of our group.
			if !(u.group.id == payload.group || is_parent) {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}
		Err(e) => {
			error!(
				message = "Could not check auth header",
				headers = ?headers,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not check auth header"),
			)
				.into_response();
		}
	}

	info!(
		message = "Received adduser request",
		payload = ?payload
	);

	match state
		.main_db
		.auth
		.new_user(&payload.username, &payload.password, payload.group.into())
		.await
	{
		Ok(()) => {
			info!(
				message = "Created user",
				payload = ?payload
			);
			return StatusCode::OK.into_response();
		}

		Err(CreateUserError::AlreadyExists) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("User `{}` already exists", payload.username),
			)
				.into_response();
		}

		Err(CreateUserError::BadName(msg)) => {
			return (StatusCode::BAD_REQUEST, format!("Invalid name: {msg}")).into_response();
		}

		Err(CreateUserError::BadGroup) => {
			return (StatusCode::BAD_REQUEST, format!("Invalid group")).into_response();
		}

		Err(CreateUserError::BadPassword) => {
			return (StatusCode::BAD_REQUEST, format!("Invalid password")).into_response();
		}

		Err(CreateUserError::DbError(e)) => {
			error!(
				message = "Could not add user",
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not add user"),
			)
				.into_response();
		}
	};
}
