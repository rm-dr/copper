use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{api::RouterState, helpers::maindb::errors::CreateGroupError};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct AddgroupRequest {
	name: String,
	parent: Option<u32>,
}

/// Create a new group
#[utoipa::path(
	post,
	path = "/group",
	responses(
		(status = 200, description = "Successfully created group"),
		(status = 400, description = "Could not create group"),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn add_group(
	State(state): State<RouterState>,
	Json(payload): Json<AddgroupRequest>,
) -> Response {
	info!(
		message = "Received addgroup request",
		payload = ?payload
	);

	match state
		.main_db
		.new_group(&payload.name, payload.parent.map(|x| x.into()))
		.await
	{
		Ok(()) => {
			info!(
				message = "Created group",
				payload = ?payload
			);
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
			return (StatusCode::BAD_REQUEST, format!("Invalid parent group")).into_response();
		}

		Err(CreateGroupError::DbError(e)) => {
			error!(
				message = "Could not create group",
				request_payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not create group"),
			)
				.into_response();
		}
	};
}
