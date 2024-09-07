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

use crate::api::RouterState;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(super) struct RenameDatasetRequest {
	pub old_name: String,
	pub new_name: String,
}

/// Delete a dataset
#[utoipa::path(
	post,
	path = "/rename",
	responses(
		(status = 200, description = "Dataset renamed successfully"),
		(status = 400, description = "Could not rename dataset", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn rename_dataset(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<RenameDatasetRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(u) => {
			if !u.group.permissions.edit_datasets.is_allowed() {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}
	}

	let res = state
		.main_db
		.dataset
		.rename_dataset(&payload.old_name, &payload.new_name)
		.await;

	match res {
		Ok(_) => {}
		Err(RenameDatasetError::BadName(err)) => {
			return (StatusCode::BAD_REQUEST, err.to_string()).into_response()
		}
		Err(RenameDatasetError::AlreadyExists) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("A dataset named `{}` already exists.", payload.new_name),
			)
				.into_response();
		}
		Err(RenameDatasetError::DbError(e)) => {
			error!(
				message = "Database error while making new dataset",
				error = ?e
			);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	return StatusCode::OK.into_response();
}
