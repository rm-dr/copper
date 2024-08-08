use axum::{
	extract::{Multipart, Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadFragmentMetadata {
	pub part_idx: u32,
	pub part_hash: String,
}

/// Upload a fragment of a file
#[utoipa::path(
	post,
	path = "/{job_id}/{file_id}",
	params(
		("job_id", description = "Upload job id"),
		("file_id", description = "Upload file name"),
	),
	responses(
		(status = 200, description = "Fragment uploaded successfully"),
		(status = 404, description = "Job or file id does not exist"),
		(status = 400, description = "Malformed request"),
		(
			status = 500,
			description = "Internal error, check server logs. Should not appear during normal operation.",
			body = String,
			example = json!("error text")
		)
	),
)]
pub(super) async fn upload(
	jar: CookieJar,
	State(state): State<RouterState>,
	Path((job_id, file_id)): Path<(String, String)>,
	mut multipart: Multipart,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let mut meta: Option<UploadFragmentMetadata> = None;
	let mut saw_data = false;

	while let Some(field) = multipart.next_field().await.unwrap() {
		let name = field.name().unwrap().to_string();

		match &name[..] {
			"metadata" => {
				if meta.is_some() {
					warn!(
						message = "Multiple `metadata` fields in a file fragment",
						job = ?job_id,
						file_id = ?file_id
					);

					return (
						StatusCode::BAD_REQUEST,
						"multiple `metadata` fields in one file fragment",
					)
						.into_response();
				}

				meta = serde_json::from_str(&field.text().await.unwrap()).unwrap();
			}

			"fragment" => {
				if saw_data {
					warn!(
						message = "Multiple `fragment` fields in a file fragment",
						job = ?job_id,
						file_id = ?file_id
					);

					return (
						StatusCode::BAD_REQUEST,
						"multiple `fragment` fields in one file fragment",
					)
						.into_response();
				}

				if meta.is_none() {
					warn!(
						message = "File fragment received before metadata",
						job = ?job_id,
						file_id = ?file_id
					);

					return (
						StatusCode::BAD_REQUEST,
						"File fragment received before metadata",
					)
						.into_response();
				}

				saw_data = true;
				let data = match field.bytes().await {
					Ok(x) => x,
					Err(_) => {
						warn!(
							message = "Failed reading fragment bytes, client probably disconnected",
							job = ?job_id,
							file_id = ?file_id
						);

						return (
							StatusCode::INTERNAL_SERVER_ERROR,
							"Failed reading fragment bytes, client probably disconnected",
						)
							.into_response();
					}
				};

				let m = meta.as_ref().unwrap();
				match state
					.uploader
					.consume_fragment(&job_id, &file_id, &data, m.part_idx, &m.part_hash)
					.await
				{
					Ok(()) => {}
					Err(e) => {
						error!(
							message = "Could not consume fragment",
							job = ?job_id,
							file_id = ?file_id,
							error = ?e
						);

						return e.into_response();
					}
				};
			}

			_ => {
				warn!(
					message = "Bad field name in fragment upload request",
					job = ?job_id,
					file_id = ?file_id,
					field_name = ?name
				);

				return (StatusCode::BAD_REQUEST, format!("bad field name `{name}`"))
					.into_response();
			}
		}
	}

	return StatusCode::OK.into_response();
}
