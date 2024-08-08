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
pub(super) struct DeleteDatasetRequest {
	/// The dataset to delete from.
	pub dataset_name: String,
}

/// Delete a dataset
#[utoipa::path(
	delete,
	path = "/del",
	responses(
		(status = 200, description = "Dataset deleted successfully"),
		(status = 400, description = "Could not delete dataset", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	)
)]
pub(super) async fn del_dataset(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<DeleteDatasetRequest>,
) -> Response {
	match state.main_db.auth.check_cookies(&jar).await {
		Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
		Ok(Some(u)) => {
			if !u.group.permissions.edit_datasets.is_allowed() {
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

	let res = state
		.main_db
		.dataset
		.del_dataset(&payload.dataset_name)
		.await;

	match res {
		Ok(_) => {}
		Err(e) => {
			error!(
				message = "Could not delete dataset",
				payload = ?payload,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete dataset `{}`", payload.dataset_name),
			)
				.into_response();
		}
	};

	return StatusCode::OK.into_response();
}
