use futures::lock::Mutex;
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use std::{path::PathBuf, time::Instant};
use tracing::{debug, error, info, warn};
use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::nodetype::UFONodeType;
use ufo_util::mime::MimeType;

use crate::config::UfodConfig;

const UPLOAD_ID_LENGTH: usize = 8;

pub(super) struct UploadJob {
	pub id: SmartString<LazyCompact>,
	pub dir: PathBuf,
	pub bound_to_pipeline_job: Option<u128>,

	pub started_at: Instant,
	pub last_activity: Instant,
	pub files: Vec<UploadJobFile>,
}

#[derive(Clone)]
pub(super) struct UploadJobFile {
	pub name: SmartString<LazyCompact>,
	pub file_type: MimeType,
	pub is_done: bool,
}

#[derive(Debug)]
pub(crate) enum JobBindError {
	/// We tried to bind a job that doesn't exist
	NoSuchJob,

	/// We tried to bind a job that has already been bound
	AlreadyBound,
}

pub(crate) struct Uploader {
	pub(super) tmp_dir: PathBuf,
	pub(super) jobs: Mutex<Vec<UploadJob>>,
}

impl Uploader {
	pub fn new(tmp_dir: PathBuf) -> Self {
		Self {
			tmp_dir,
			jobs: Mutex::new(Vec::new()),
		}
	}

	#[inline(always)]
	pub(super) fn generate_id() -> SmartString<LazyCompact> {
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
	pub async fn check_jobs(&self, config: &UfodConfig, runner: &PipelineRunner<UFONodeType>) {
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
				config.delete_job_after_bound
			} else {
				config.delete_job_after_unbound
			};

			// Wait for timeout even if this job is bound,
			// just in case it has been created but hasn't yet been added to the runner.
			if j.last_activity + offset < now {
				if j.bound_to_pipeline_job.is_none() {
					info!(message = "Removing job", reason = "timeout", job_id = ?j.id);
				} else {
					info!(message = "Removing job", reason = "pipeline is done", job_id = ?j.id);
				}

				let j = jobs.swap_remove(i);
				match std::fs::remove_dir_all(&j.dir) {
					Ok(()) => {}
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

		let job = jobs.iter().find(|us| us.id == *job_id)?;
		let file = job.files.iter().find(|f| f.name == *file_name)?;

		return Some(job.dir.join(file.name.as_str()));
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
		return Some(file.is_done);
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
		upload_job_id: &SmartString<LazyCompact>,
		pipeline_job_id: u128,
	) -> Result<(), JobBindError> {
		let mut jobs = self.jobs.lock().await;

		// Try to find the given job
		let job = if let Some(x) = jobs.iter_mut().find(|us| us.id == *upload_job_id) {
			x
		} else {
			warn!(
				message = "Tried to bind job that doesn't exist",
				job = ?upload_job_id,
				pipeline = pipeline_job_id
			);
			return Err(JobBindError::NoSuchJob);
		};

		if job.bound_to_pipeline_job.is_some() {
			warn!(
				message = "Tried to bind job, but it is alredy bound",
				job = ?job.id,
				pipeline = pipeline_job_id
			);
			return Err(JobBindError::AlreadyBound);
		}

		job.bound_to_pipeline_job = Some(pipeline_job_id);
		info!(
			message = "Bound job to pipeline",
			upload_job = ?job.id,
			pipeline_job = pipeline_job_id
		);

		return Ok(());
	}
}
