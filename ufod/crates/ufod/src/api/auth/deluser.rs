use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct DeluserRequest {
	user: u32,
}

/// Delete a user
#[utoipa::path(
	delete,
	path = "/user",
	responses(
		(status = 200, description = "Successfully deleted user"),
		(status = 400, description = "Could not delete user"),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn del_user(
	State(state): State<RouterState>,
	Json(payload): Json<DeluserRequest>,
) -> Response {
	info!(
		message = "Received deluser request",
		payload = ?payload
	);

	match state.main_db.del_user(payload.user.into()).await {
		Ok(()) => {
			info!(
				message = "Deleted user",
				payload = ?payload
			);
			return StatusCode::OK.into_response();
		}

		Err(e) => {
			error!(
				message = "Could not delete user",
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete user"),
			)
				.into_response();
		}
	};
}
