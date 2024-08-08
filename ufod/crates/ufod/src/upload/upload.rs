use std::{fs::File, io::Write, sync::Arc, time::Instant};

use axum::{
	extract::{Multipart, Path},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use utoipa::ToSchema;

use super::uploader::Uploader;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct UploadFragmentMetadata {
	pub part_idx: u32,
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
	uploader: Arc<Uploader>,
	Path((job_id, file_id)): Path<(String, String)>,
	mut multipart: Multipart,
) -> Response {
	let mut jobs = uploader.jobs.lock().await;

	// Try to find the given job
	let job = match jobs.iter_mut().find(|us| us.id == job_id) {
		Some(x) => x,
		None => {
			warn!(
				message = "Tried to upload a fragment to a job that doesn't exist",
				bad_job_id = ?job_id,
			);

			return (
				StatusCode::NOT_FOUND,
				format!("upload job {job_id} does not exist"),
			)
				.into_response();
		}
	};
	job.last_activity = Instant::now();

	// Try to find the given file
	let file = match job.files.iter_mut().find(|f| f.name == file_id) {
		Some(x) => x,
		None => {
			warn!(
				message = "Tried to upload a fragment to a file that doesn't exist",
				job = ?job_id,
				bad_file_id = ?file_id
			);

			return (
				StatusCode::NOT_FOUND,
				format!("upload job {job_id} does have a file with id {file_id}"),
			)
				.into_response();
		}
	};

	if file.is_done {
		warn!(
			message = "Tried to upload a fragment to a file that has been finished",
			job = ?job_id,
			file_id = ?file_id
		);

		return (
			StatusCode::BAD_REQUEST,
			format!("file {} has already been finished", file_id),
		)
			.into_response();
	}

	// Release lock.
	// We don't want to hold this while processing large files.
	// TODO: don't expire files while we're here
	let job_dir = job.dir.clone();
	let file = file.clone();
	drop(jobs);

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

				// TODO: better organize these errors
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
				let m = meta.as_ref().unwrap();
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

				// TODO: consistently name "frag"
				let frag_path = job_dir.join(format!("{}-frag-{}", file.name, m.part_idx));

				let mut f = match File::create(&frag_path) {
					Ok(f) => f,
					Err(e) => {
						error!(
							message = "Could not create fragment file",
							job = ?job_id,
							file_id = ?file_id,
							frag_path = ?frag_path,
							error = ?e
						);

						return (
							StatusCode::INTERNAL_SERVER_ERROR,
							format!(
								"could not create fragment {} of file {} in job {}",
								m.part_idx, file_id, job_id
							),
						)
							.into_response();
					}
				};

				match f.write(&data) {
					Ok(_) => {}
					Err(e) => {
						error!(
							message = "Could not write fragment to file",
							job = ?job_id,
							file_id = ?file_id,
							frag_path = ?frag_path,
							error = ?e
						);

						return (
							StatusCode::INTERNAL_SERVER_ERROR,
							format!("could not append to file {} in job {}", file_id, job_id),
						)
							.into_response();
					}
				}
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
