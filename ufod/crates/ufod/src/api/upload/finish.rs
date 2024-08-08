use std::{
	fs::File,
	io::{Read, Write},
	sync::Arc,
	time::Instant,
};

use axum::{
	extract::Path,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use smartstring::{LazyCompact, SmartString};
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::uploader::Uploader;

// TODO: send finish progress

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
	uploader: Arc<Uploader>,
	Path((job_id, file_id)): Path<(String, String)>,
	Json(finish_data): Json<UploadFinish>,
) -> Response {
	let mut jobs = uploader.jobs.lock().await;

	// Try to find the given job
	let job = match jobs.iter_mut().find(|us| us.id == job_id) {
		Some(x) => x,
		None => {
			warn!(
				message = "Tried to finish a file in a job that doesn't exist",
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
				message = "Tried to finish a file that doesn't exist",
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

	let final_file_path = job.dir.join(file.name.as_str());
	let mut final_file = File::create(final_file_path).unwrap();

	let n_frags = finish_data.frag_count;
	let mut hasher = Sha256::new();
	for frag_idx in 0..n_frags {
		let frag_path = job.dir.join(format!("{}-frag-{frag_idx}", file.name));
		if !frag_path.is_file() {
			warn!(
				message = "Tried to finish file with missing fragment",
				job = ?job_id,
				file = ?file_id,
				expected_frag=n_frags,
				missing_frag=frag_idx
			);

			return (
				StatusCode::BAD_REQUEST,
				format!(
					"Tried to finish file with missing fragment (idx {})",
					frag_idx
				),
			)
				.into_response();
		}

		let mut f = File::open(&frag_path).unwrap();
		let mut data = Vec::new();
		f.read_to_end(&mut data).unwrap();
		hasher.update(&data);
		final_file.write_all(&data).unwrap();
		drop(f);
		std::fs::remove_file(frag_path).unwrap();
	}

	file.is_done = true;
	let our_hash = format!("{:X}", hasher.finalize());

	if our_hash != finish_data.hash {
		warn!(
			message = "Uploaded hash does not match expected hash",
			job = ?job_id,
			file = ?file_id,
			expected_hash = ?finish_data.hash,
			got_hash = ?our_hash
		);
	}
	info!(
		message = "Finished uploading file",
		job = ?job_id,
		file = ?file_id,
		hash = ?our_hash,
		file_type = ?file.file_type,
	);

	return StatusCode::OK.into_response();

	/*
	if our_hash != finish_data.hash {
		warn!(
			message = "Uploaded hash does not match expected hash",
			job = ?job_id,
			file = ?file_id,
			expected_hash = ?finish_data.hash,
			got_hash = ?our_hash
		);

		return (
			StatusCode::BAD_REQUEST,
			format!(
				"uploaded file hash `{}` does not match expected hash `{}`",
				our_hash, finish_data.hash
			),
		)
			.into_response();
	} else {
		info!(
			message = "Finished uploading file",
			job = ?job_id,
			file = ?file_id,
			hash = ?our_hash
		);

		return StatusCode::OK.into_response();
	}
	*/
}
