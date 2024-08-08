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
pub(super) struct AddgroupRequest {
	name: String,
	parent: GroupId,
}

/// Create a new group
#[utoipa::path(
	post,
	path = "/group/add",
	responses(
		(status = 200, description = "Successfully created group"),
		(status = 400, description = "Could not create group"),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn add_group(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<AddgroupRequest>,
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
				.is_group_parent(u.group.id, payload.parent)
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
			if !(u.group.id == payload.parent || is_parent) {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}
	}

	match state
		.main_db
		.auth
		.new_group(&payload.name, payload.parent)
		.await
	{
		Ok(()) => {
			return StatusCode::OK.into_response();
		}

		Err(CreateGroupError::AlreadyExists) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Group `{}` already exists", payload.name),
			)
				.into_response();
		}

		Err(CreateGroupError::BadName(msg)) => {
			return (StatusCode::BAD_REQUEST, format!("Invalid name: {msg}")).into_response();
		}

		Err(CreateGroupError::BadParent) => {
			return (StatusCode::BAD_REQUEST, "Invalid parent group").into_response();
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
