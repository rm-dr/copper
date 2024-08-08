use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{sync::Arc, time::Instant};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::helpers::uploader::{UploadJob, Uploader};

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
		)
	),
)]

pub(super) async fn start_upload(uploader: Arc<Uploader>) -> Response {
	let mut jobs = uploader.jobs.lock().await;

	let id = loop {
		let id = Uploader::generate_id();
		if jobs.iter().all(|us| us.id != id) {
			break id;
		}
	};

	let upload_job_dir = uploader.config.paths.upload_dir.join(id.to_string());
	match std::fs::create_dir(&upload_job_dir) {
		Ok(_) => {}
		Err(e) => {
			error!(
				message = "Could not create upload job",
				job = ?id,
				error = ?e
			);

			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("could not create directory for upload job `{id}`"),
			)
				.into_response();
		}
	}

	let now = Instant::now();
	jobs.push(UploadJob {
		id: id.clone(),
		dir: upload_job_dir,
		started_at: now.clone(),
		last_activity: now,
		files: Vec::new(),
		bound_to_pipeline_job: None,
	});

	info!(message = "Created upload job", job=?id);
	return (StatusCode::OK, Json(UploadStartResult { job_id: id })).into_response();
}
