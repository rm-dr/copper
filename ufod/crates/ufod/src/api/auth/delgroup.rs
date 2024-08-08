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
	),
)]
pub(super) async fn del_group(
	State(state): State<RouterState>,
	Json(payload): Json<DelgroupRequest>,
) -> Response {
	info!(
		message = "Received delgroup request",
		payload = ?payload
	);

	match state.main_db.del_group(payload.group.into()).await {
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
