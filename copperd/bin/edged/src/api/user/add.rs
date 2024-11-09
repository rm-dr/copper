use crate::database::base::{client::DatabaseClient, errors::user::AddUserError};
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use copper_edged::UserPassword;
use copper_itemdb::client::base::client::ItemdbClient;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NewUserRequest {
	name: String,
	email: String,
	password: String,
}

/// Create a new User
#[utoipa::path(
	post,
	path = "",
	responses(
		(status = 200, description = "User created successfully"),
		(status = 400, description = "Bad request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn add_user<Client: DatabaseClient, Itemdb: ItemdbClient>(
	State(state): State<RouterState<Client, Itemdb>>,
	Json(payload): Json<NewUserRequest>,
) -> Response {
	let password = UserPassword::new(&payload.password);
	let res = state
		.db_client
		.add_user(&payload.email, &payload.name, &password)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(AddUserError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(AddUserError::UniqueEmailViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a user with this email already exists"),
		)
			.into_response(),

		Err(AddUserError::DbError(e)) => {
			error!(
				message = "Database error while making new user",
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
