use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{api::RouterState, helpers::maindb::errors::CreateUserError};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct AdduserRequest {
	username: String,
	password: String,
	group: u32,
}

/// Create a new user
#[utoipa::path(
	post,
	path = "/user",
	responses(
		(status = 200, description = "Successfully created user"),
		(status = 400, description = "Could not create user"),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn add_user(
	State(state): State<RouterState>,
	Json(payload): Json<AdduserRequest>,
) -> Response {
	info!(
		message = "Received adduser request",
		payload = ?payload
	);

	match state
		.main_db
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
