use std::{
	fs::{File, OpenOptions},
	io::Write,
	path::PathBuf,
	sync::Arc,
	time::{Duration, Instant},
};

use axum::{
	extract::{Multipart, Path},
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::post,
	Json, Router,
};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use smartstring::{LazyCompact, SmartString};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use ufo_api::upload::{
	UploadFinish, UploadFragmentMetadata, UploadNewFileResult, UploadStartInfo, UploadStartResult,
};
use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::nodetype::UFONodeType;
use ufo_util::mime::MimeType;

use crate::RouterState;

// TODO: better error handling
// TODO: delete when fail
// TODO: logging

const UPLOAD_ID_LENGTH: usize = 8;

struct UploadJob {
	id: SmartString<LazyCompact>,
	dir: PathBuf,
	bound_to_pipeline_job: Option<u128>,

	started_at: Instant,
	last_activity: Instant,
	files: Vec<UploadJobFile>,
}

struct UploadJobFile {
	name: SmartString<LazyCompact>,
	path: PathBuf,
	file_type: MimeType,

	fragments_received: u32,
	is_done: bool,
	hasher: Option<Sha256>,
}

pub(crate) struct Uploader {
	tmp_dir: PathBuf,
	jobs: Mutex<Vec<UploadJob>>,
	delete_job_after_bound: Duration,
	delete_job_after_unbound: Duration,
}

#[derive(Debug)]
pub enum JobBindError {
	/// We tried to bind a job that doesn't exist
	NoSuchJob,

	/// We tried to bind a job that has already been bound
	AlreadyBound,
}

impl Uploader {
	pub fn new(tmp_dir: PathBuf) -> Self {
		Self {
			tmp_dir,
			jobs: Mutex::new(Vec::new()),
			delete_job_after_unbound: Duration::from_secs(5),
			delete_job_after_bound: Duration::from_secs(10),
		}
	}

	pub fn get_router(uploader: Arc<Self>) -> Router<RouterState> {
		let mut r = Router::new();

		let u = uploader.clone();
		r = r.route(
			"/start",
			post(|| async move { Self::start_upload(u).await }),
		);

		let u = uploader.clone();
		r = r.route(
			"/:job_id/new_file",
			post(|path, payload| async move { Self::start_file(u, path, payload).await }),
		);

		let u = uploader.clone();
		r = r.route(
			"/:job_id/:file_handle",
			post(|path, multipart| async move { Self::upload(u, path, multipart).await }),
		);

		let u = uploader.clone();
		r = r.route(
			"/:job_id/:file_id/finish",
			post(|path, payload| async move { Self::finish_file(u, path, payload).await }),
		);

		return r;
	}

	#[inline(always)]
	fn generate_id() -> SmartString<LazyCompact> {
		rand::thread_rng()
			.sample_iter(&Alphanumeric)
			.take(UPLOAD_ID_LENGTH)
			.map(char::from)
			.collect()
	}

	/// Check all active jobs in this uploader,
	/// and remove jobs we no longer need.
	///
	/// This cleans up jobs that have timed out,
	/// and jobs bound to a pipeline that has been finished.
	pub async fn check_jobs(&self, runner: &PipelineRunner<UFONodeType>) {
		let mut jobs = self.jobs.lock().await;

		let now = Instant::now();
		let mut i = 0;
		while i < jobs.len() {
			let j = &jobs[i];

			if let Some(p) = j.bound_to_pipeline_job {
				let is_active = runner.active_job_by_id(p).is_some();
				let is_pending = runner.active_job_by_id(p).is_some();

				// Not active and not pending implies done
				if is_active || is_pending {
					i += 1;
					continue;
				}
			}

			let offset = if j.bound_to_pipeline_job.is_some() {
				self.delete_job_after_bound
			} else {
				self.delete_job_after_unbound
			};

			// Wait for timeout even if this job is bound,
			// just in case it has been created but hasn't yet been added to the runner.
			if j.last_activity + offset < now {
				if j.bound_to_pipeline_job.is_none() {
					debug!(message = "Removing job", reason = "timeout", job_id = ?j.id);
				} else {
					debug!(message = "Removing job", reason = "pipeline is done", job_id = ?j.id);
				}

				let j = jobs.swap_remove(i);
				match std::fs::remove_dir_all(&j.dir) {
					Ok(()) => {
						info!(message = "Removed job directory", job_id = ?j.id, path = ?j.dir)
					}
					Err(e) => {
						error!(message = "Failed removing job directory", job_id = ?j.id, path = ?j.dir, error=?e)
					}
				}

				continue;
			}

			i += 1;
		}
	}
}

impl Uploader {
	/// Get a path to the given file
	pub async fn get_job_file_path(
		&self,
		job_id: &SmartString<LazyCompact>,
		file_name: &SmartString<LazyCompact>,
	) -> Option<PathBuf> {
		let jobs = self.jobs.lock().await;

		// Try to find the given job
		let job = jobs.iter().find(|us| us.id == *job_id)?;

		// Try to find the given file
		let file = job.files.iter().find(|f| f.name == *file_name)?;

		return Some(file.path.clone());
	}

	/// Has the given file been finished?
	pub async fn has_file_been_finished(
		&self,
		job_id: &SmartString<LazyCompact>,
		file_name: &SmartString<LazyCompact>,
	) -> Option<bool> {
		let jobs = self.jobs.lock().await;

		let job = jobs.iter().find(|us| us.id == *job_id)?;
		let file = job.files.iter().find(|f| f.name == *file_name)?;
		return Some(file.hasher.is_none());
	}

	/// Bind the given job to the given pipeline.
	///
	/// This ensures that this job's files will removed only after
	/// this pipeline finishes running.
	///
	/// Notes:
	/// - Unbound jobs are removed after a preset duration of inactivity.
	/// - Any job may only be bound to one pipeline.
	/// - Once a job is bound, it cannot be bound again.
	pub async fn bind_job_to_pipeline(
		&self,
		job_id: &SmartString<LazyCompact>,
		pipeline_id: u128,
	) -> Result<(), JobBindError> {
		let mut jobs = self.jobs.lock().await;

		// Try to find the given job
		let job = if let Some(x) = jobs.iter_mut().find(|us| us.id == *job_id) {
			x
		} else {
			warn!(
				message = "Tried to bind job that doesn't exist",
				job = ?job_id,
				pipeline = pipeline_id
			);
			return Err(JobBindError::NoSuchJob);
		};

		if job.bound_to_pipeline_job.is_some() {
			warn!(
				message = "Tried to bind job, but it is alredy bound",
				job = ?job.id,
				pipeline = pipeline_id
			);
			return Err(JobBindError::AlreadyBound);
		}

		job.bound_to_pipeline_job = Some(pipeline_id);
		debug!(
			message = "Bound job to pipeline",
			job = ?job.id,
			pipeline = pipeline_id
		);

		return Ok(());
	}
}

impl Uploader {
	/// Start an upload job and return its handle
	async fn start_upload(uploader: Arc<Self>) -> Response {
		let mut jobs = uploader.jobs.lock().await;

		let id = loop {
			let id = Self::generate_id();
			if jobs.iter().all(|us| us.id != id) {
				break id;
			}
		};

		let upload_job_dir = uploader.tmp_dir.join(id.to_string());
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

		debug!(message = "Created upload job", job=?id);
		return (StatusCode::OK, Json(UploadStartResult { job_id: id })).into_response();
	}

	/// Start a file inside an upload job and return its handle
	async fn start_file(
		uploader: Arc<Self>,
		Path(upload_job_id): Path<SmartString<LazyCompact>>,
		Json(start_info): Json<UploadStartInfo>,
	) -> Response {
		let mut jobs = uploader.jobs.lock().await;

		// Try to find the given job
		let job = match jobs.iter_mut().find(|us| us.id == upload_job_id) {
			Some(x) => x,
			None => {
				warn!(
					message = "Tried to start a file in a job that doesn't exist",
					bad_job_id = ?upload_job_id,
				);

				return (
					StatusCode::NOT_FOUND,
					format!("upload job {upload_job_id} does not exist"),
				)
					.into_response();
			}
		};
		job.last_activity = Instant::now();

		// Make a new handle for this file
		let file_name = loop {
			let id = Self::generate_id();
			if job.files.iter().all(|us| us.name != id) {
				break format!("{}{}", id, start_info.file_type.extension());
			}
		};

		// Create the file
		let file_path = job.dir.join(&file_name);
		match File::create(&file_path) {
			Ok(_) => {}
			Err(e) => {
				error!(
					message = "Could not initialize file in upload job",
					job = ?job.id,
					error = ?e,
				);

				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("could not create file {file_name} for upload job {upload_job_id}"),
				)
					.into_response();
			}
		}

		job.files.push(UploadJobFile {
			name: file_name.clone().into(),
			path: file_path,
			file_type: start_info.file_type.clone(),

			fragments_received: 0,
			is_done: false,
			hasher: Some(Sha256::new()),
		});

		debug!(
			message = "Created a new upload file",
			job = ?job.id,
			file_name= ?file_name,
			file_type = ?start_info.file_type
		);

		return (
			StatusCode::OK,
			Json(UploadNewFileResult {
				file_name: file_name.into(),
			}),
		)
			.into_response();
	}

	async fn upload(
		uploader: Arc<Self>,
		Path((job_id, file_id)): Path<(SmartString<LazyCompact>, SmartString<LazyCompact>)>,
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

		let mut saw_meta = false;
		let mut saw_data = false;

		while let Some(field) = multipart.next_field().await.unwrap() {
			let name = field.name().unwrap().to_string();

			match &name[..] {
				"metadata" => {
					if saw_meta {
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

					saw_meta = true;
					let meta = field.text().await.unwrap();
					let meta: UploadFragmentMetadata = serde_json::from_str(&meta).unwrap();

					if file.fragments_received != meta.part_idx {
						warn!(
							message = "Bad fragment index",
							job = ?job_id,
							file_id = ?file_id,
							expected_idx = file.fragments_received,
							got_idx = meta.part_idx,
						);

						return (
							StatusCode::BAD_REQUEST,
							format!(
								"bad fragment index: expected {}, got {}",
								file.fragments_received, meta.part_idx
							),
						)
							.into_response();
					}

					file.fragments_received += 1;
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

					saw_data = true;
					let data = field.bytes().await.unwrap();
					file.hasher.as_mut().unwrap().update(data.clone());

					let f = OpenOptions::new()
						.create(false)
						.append(true)
						.open(&file.path);

					match f {
						Ok(mut f) => match f.write(&data) {
							Ok(_) => {}
							Err(e) => {
								error!(
									message = "Could not write fragment to file",
									job = ?job_id,
									file_id = ?file_id,
									file_path = ?file.path,
									error = ?e
								);

								return (
									StatusCode::INTERNAL_SERVER_ERROR,
									format!(
										"could not append to file {} in job {}",
										file_id, job_id
									),
								)
									.into_response();
							}
						},
						Err(e) => {
							error!(
								message = "Could not open file to write fragment",
								job = ?job_id,
								file_id = ?file_id,
								file_path = ?file.path,
								error = ?e
							);

							return (
								StatusCode::INTERNAL_SERVER_ERROR,
								format!("could not open file {} in job {}", file_id, job_id),
							)
								.into_response();
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

	async fn finish_file(
		uploader: Arc<Self>,
		Path((job_id, file_id)): Path<(SmartString<LazyCompact>, SmartString<LazyCompact>)>,
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

		file.is_done = true;
		let our_hash = format!("{:X}", file.hasher.take().unwrap().finalize());

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
			debug!(
				message = "Finished uploading file",
				job = ?job_id,
				file = ?file_id,
				hash = ?our_hash
			);

			return StatusCode::OK.into_response();
		}
	}
}