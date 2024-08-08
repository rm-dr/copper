use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::warn;
use utoipa::ToSchema;

use crate::api::RouterState;

/// Parameters to finish an uploading file
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadFinish {
	/// The total number of fragments this file should have
	pub frag_count: u32,

	/// The hash of this complete file
	#[schema(value_type = String)]
	pub hash: SmartString<LazyCompact>,
}

/// Finish a file upload
#[utoipa::path(
	post,
	path = "/{job_id}/{file_id}/finish",
	params(
		("job_id", description = "Upload job id"),
		("file_id", description = "Upload file name"),
	),
	responses(
		(status = 200, description = "File finished successfully", body = ()),
		(status = 404, description = "Bad job or file id"),
		(status = 400, description = "Malformed request or unfinished upload"),
		(
			status = 500,
			description = "Internal error, check server logs. Should not appear during normal operation.",
			body = String,
			example = json!("error text")
		)
	)
)]
pub(super) async fn finish_file(
	jar: CookieJar,
	State(state): State<RouterState>,
	Path((job_id, file_id)): Path<(String, String)>,
	Json(finish_data): Json<UploadFinish>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let t_job_id = job_id.clone();
	let t_file_id = file_id.clone();
	let t_finish_hash = finish_data.hash.clone();
	match tokio::task::spawn_blocking(move || {
		state.uploader.finish_file(
			&t_job_id,
			&t_file_id,
			finish_data.frag_count,
			&t_finish_hash,
		)
	})
	.await
	{
		Ok(Ok(())) => {
			return StatusCode::OK.into_response();
		}

		Err(e) => {
			warn!(
				message = "spawn_blocking exited with error",
				job_id = ?job_id,
				file_id = ?file_id,
				error = ?e
			);

			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"spawn_blocking exited with error",
			)
				.into_response();
		}

		Ok(Err(e)) => {
			warn!(
				message = "Could not finish uploading file",
				job_id = ?job_id,
				file_id = ?file_id,
				error = ?e
			);

			return e.into_response();
		}
	};
}
