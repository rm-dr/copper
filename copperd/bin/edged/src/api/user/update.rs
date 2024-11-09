use crate::database::base::{client::DatabaseClient, errors::user::UpdateUserError};
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_edged::UserPassword;
use copper_itemdb::client::base::client::ItemdbClient;
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct UpdateUserRequest {
	pub new_name: Option<String>,
	pub new_email: Option<String>,
	pub new_password: Option<String>,
}

/// Update a user
#[utoipa::path(
	patch,
	path = "/{user_id}",
	params(
		("user_id", description = "User id"),
	),
	responses(
		(status = 200, description = "User updated successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn update_user<Client: DatabaseClient, Itemdb: ItemdbClient>(
	// OriginalUri(uri): OriginalUri,
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(user_id): Path<i64>,
	Json(payload): Json<UpdateUserRequest>,
) -> Response {
	let mut user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	// Users can only update themselves.
	if user.id != user_id.into() {
		return (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response();
	}

	// Update user info
	if let Some(name) = payload.new_name {
		user.name = name.into()
	}

	if let Some(email) = payload.new_email {
		user.email = email.into()
	}

	if let Some(pass) = payload.new_password {
		user.password = UserPassword::new(&pass);
	}

	// Save user info
	let res = state.db_client.update_user(&user).await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(UpdateUserError::UniqueEmailViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a user with this email already exists"),
		)
			.into_response(),

		Err(UpdateUserError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(UpdateUserError::DbError(e)) => {
			error!(
				message = "Database error while renaming user",
				error = ?e
			);

			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response()
		}
	};
}
