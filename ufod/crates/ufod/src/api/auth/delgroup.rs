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

use crate::api::RouterState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct DelgroupRequest {
	group: u32,
}

/// Delete a group
#[utoipa::path(
	delete,
	path = "/group",
	responses(
		(status = 200, description = "Successfully deleted group"),
		(status = 400, description = "Could not delete group"),
		(status = 500, description = "Internal server error", body=String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn del_group(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<DelgroupRequest>,
) -> Response {
	match state.main_db.auth.check_cookies(&jar).await {
		Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
		Ok(Some(u)) => {
			if !u.group.permissions.edit_groups.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}

			// Is the group we want to delete a child of this user's group?
			let is_parent = match state
				.main_db
				.auth
				.is_group_parent(u.group.id, payload.group.into())
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
						format!("Could not check group parent"),
					)
						.into_response();
				}
			};

			// We can only create groups that are children of our group.
			// Node that users may NOT delete their own group.
			if !is_parent {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}
		Err(e) => {
			error!(
				message = "Could not check auth cookies",
				cookies = ?jar,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not check auth cookies"),
			)
				.into_response();
		}
	}

	info!(
		message = "Received delgroup request",
		payload = ?payload
	);

	match state.main_db.auth.del_group(payload.group.into()).await {
		Ok(()) => {
			info!(
				message = "Deleted group",
				payload = ?payload
			);
			return StatusCode::OK.into_response();
		}

		Err(e) => {
			error!(
				message = "Could not delete group",
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete group"),
			)
				.into_response();
		}
	};
}
