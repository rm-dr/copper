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
	maindb::auth::{errors::CreateGroupError, GroupId},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct RenameGroupRequest {
	group: GroupId,
	new_name: String,
}

/// Create a new user
#[utoipa::path(
	post,
	path = "/group/rename",
	responses(
		(status = 200, description = "Successfully renamed group"),
		(status = 400, description = "Could not rename group"),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn rename_group(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<RenameGroupRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => {
			if !u.group.permissions.edit_groups.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			// Is the group we want to create a child of this user's group?
			let is_parent = match state
				.main_db
				.auth
				.is_group_parent(u.group.id, payload.group)
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

			// We can only create groups that are children of our group,
			// or children of subgroups of our group.
			if !(u.group.id == payload.group || is_parent) {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}
	}

	match state
		.main_db
		.auth
		.rename_group(payload.group, &payload.new_name)
		.await
	{
		Ok(()) => {
			return StatusCode::OK.into_response();
		}

		Err(CreateGroupError::AlreadyExists) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Group `{}` already exists", payload.new_name),
			)
				.into_response();
		}

		Err(CreateGroupError::BadName(err)) => {
			return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
		}

		Err(CreateGroupError::BadParent) => {
			unreachable!()
		}

		Err(CreateGroupError::DbError(e)) => {
			error!(
				message = "Could not create group",
				request_payload = ?payload,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not create group").into_response();
		}
	};
}
