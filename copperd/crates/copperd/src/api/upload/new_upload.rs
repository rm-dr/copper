use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::{error, warn};
use utoipa::ToSchema;

use crate::api::RouterState;

/// A freshly-started upload job's parameters
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadStartResult {
	/// This upload job's id
	#[schema(value_type = String)]
	pub job_id: SmartString<LazyCompact>,
}

/// Start an upload job and return its handle
#[utoipa::path(
	post,
	path = "/new",
	responses(
		(status = 200, description = "New upload info", body = UploadStartResult),
		(
			status = 500,
			description = "Internal error, check server logs. Should not appear during normal operation.",
			body = String,
			example = json!("error text")
		),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn start_upload(jar: CookieJar, State(state): State<RouterState>) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	match tokio::task::spawn_blocking(move || state.uploader.new_job()).await {
		Ok(Ok(id)) => {
			return (StatusCode::OK, Json(UploadStartResult { job_id: id })).into_response();
		}

		Err(e) => {
			warn!(
				message = "spawn_blocking exited with error",
				error = ?e
			);

			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"spawn_blocking exited with error",
			)
				.into_response();
		}

		Ok(Err(e)) => {
			error!(
				message = "Could not create upload job",
				error = ?e
			);

			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"could not create upload job",
			)
				.into_response();
		}
	}
}