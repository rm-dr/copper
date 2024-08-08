use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use smartstring::{LazyCompact, SmartString};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, sync::Arc, time::Instant};
use tracing::{error, info, warn};
use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::nodetype::UFONodeType;
use ufo_util::mime::MimeType;

use crate::config::UfodConfig;

pub mod errors;

const UPLOAD_ID_LENGTH: usize = 8;

pub struct UploadJob {
	pub id: SmartString<LazyCompact>,
	pub dir: PathBuf,
	pub bound_to_pipeline_job: Option<u128>,

	pub started_at: Instant,
	pub last_activity: Instant,
	pub files: Vec<UploadJobFile>,
}

#[derive(Debug, Copy, Clone)]
pub enum UploadJobFileState {
	/// We're waiting for fragments
	Pending,

	/// User has triggered finish, finish is still running
	Finishing,

	/// File has been finished and is ready for use
	Done,
}

#[derive(Clone)]
pub struct UploadJobFile {
	pub name: SmartString<LazyCompact>,
	pub mime: MimeType,
	pub state: UploadJobFileState,
	pub frag_hashes: HashMap<String, String>,
}

pub struct Uploader {
	pub config: Arc<UfodConfig>,
	pub jobs: tokio::sync::Mutex<Vec<UploadJob>>,
}

impl Uploader {
	pub fn open(config: Arc<UfodConfig>) -> Self {
		// Initialize upload dir
		if !config.paths.upload_dir.exists() {
			info!(
				message = "Creating upload dir because it doesn't exist",
				upload_dir = ?config.paths.upload_dir
			);
			std::fs::create_dir_all(&config.paths.upload_dir).unwrap();
		} else if config.paths.upload_dir.is_dir() {
			warn!(
				message = "Upload directory isn't empty, removing",
				directory = ?config.paths.upload_dir
			);
			std::fs::remove_dir_all(&config.paths.upload_dir).unwrap();
			std::fs::create_dir_all(&config.paths.upload_dir).unwrap();
		} else {
			error!(
				message = "Upload dir is not a directory",
				upload_path = ?config.paths.upload_dir
			);
			panic!(
				"Upload dir {:?} is not a directory",
				config.paths.upload_dir
			)
		}

		Self {
			config,
			jobs: tokio::sync::Mutex::new(Vec::new()),
		}
	}

	#[inline(always)]
	pub fn generate_id() -> SmartString<LazyCompact> {
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

			if j.files
				.iter()
				.any(|f| matches!(f.state, UploadJobFileState::Finishing))
			{
				// If any files are being finished in a job,
				// it should not time out.
				continue;
			}

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
				self.config.upload.job_timeout_bound
			} else {
				self.config.upload.job_timeout_unbound
			};

			// Wait for timeout even if this job is bound,
			// just in case it has been created but hasn't yet been added to the runner.
			if j.last_activity + offset < now {
				if j.bound_to_pipeline_job.is_none() {
					info!(
						message = "Removing job",
						reason = "timeout",
						job_id = ?j.id,
						started_at = ?j.started_at
					);
				} else {
					info!(
						message = "Removing job",
						reason = "pipeline is done",
						job_id = ?j.id,
						started_at = ?j.started_at
					);
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
		return Some(matches!(file.state, UploadJobFileState::Done));
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
	) -> Result<(), errors::JobBindError> {
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
			return Err(errors::JobBindError::NoSuchJob);
		};

		if job.bound_to_pipeline_job.is_some() {
			warn!(
				message = "Tried to bind job, but it is alredy bound",
				job = ?job.id,
				pipeline = pipeline_job_id
			);
			return Err(errors::JobBindError::AlreadyBound);
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

impl Uploader {
	pub fn new_job(&self) -> Result<SmartString<LazyCompact>, std::io::Error> {
		let mut jobs = self.jobs.blocking_lock();

		let id = loop {
			let id = Uploader::generate_id();
			if jobs.iter().all(|us| us.id != id) {
				break id;
			}
		};

		let upload_job_dir = self.config.paths.upload_dir.join(id.to_string());
		std::fs::create_dir(&upload_job_dir)?;

		let now = Instant::now();
		jobs.push(UploadJob {
			id: id.clone(),
			dir: upload_job_dir,
			started_at: now.clone(),
			last_activity: now,
			files: Vec::new(),
			bound_to_pipeline_job: None,
		});

		info!(
			message = "Created a new upload job",
			job_id = ?id,
		);

		return Ok(id);
	}

	pub fn new_file(
		&self,
		job_id: &str,
		mime: MimeType,
	) -> Result<SmartString<LazyCompact>, errors::UploadNewFileError> {
		let mut jobs = self.jobs.blocking_lock();

		let job = jobs
			.iter_mut()
			.find(|us| us.id == job_id)
			.ok_or(errors::UploadNewFileError::BadUploadJob)?;
		job.last_activity = Instant::now();

		// Make a new handle for this file
		let file_id: SmartString<LazyCompact> = loop {
			let id = Uploader::generate_id();
			if job.files.iter().all(|us| us.name != id) {
				break format!("{}{}", id, mime.extension()).into();
			}
		};

		job.files.push(UploadJobFile {
			name: file_id.clone().into(),
			mime: mime.clone(),
			state: UploadJobFileState::Pending,
			frag_hashes: HashMap::new(),
		});

		info!(
			message = "Created a new upload file",
			job_id = ?job.id,
			file_id = ?file_id,
			file_type = ?mime
		);

		return Ok(file_id);
	}

	pub fn consume_fragment(
		&self,
		job_id: &str,
		file_id: &str,
		data: &[u8],
		frag_idx: u32,
		frag_hash: &str,
	) -> Result<(), errors::UploadFragmentError> {
		let mut jobs = self.jobs.blocking_lock();

		let job = jobs
			.iter_mut()
			.find(|us| us.id == job_id)
			.ok_or(errors::UploadFragmentError::BadUploadJob)?;
		job.last_activity = Instant::now();

		let file = job
			.files
			.iter_mut()
			.find(|f| f.name == file_id)
			.ok_or(errors::UploadFragmentError::BadFileID)?;

		if !matches!(file.state, UploadJobFileState::Pending) {
			return Err(errors::UploadFragmentError::AlreadyFinished);
		}

		// Release lock, we don't need it anymore
		let frag_file_name = format!("{}-frag-{}", file.name, frag_idx);
		let frag_path = job.dir.join(&frag_file_name);
		file.frag_hashes.insert(frag_file_name, frag_hash.into());
		drop(jobs);

		let mut f = File::create(&frag_path)?;
		f.write(&data)?;

		return Ok(());
	}

	pub fn finish_file(
		&self,
		job_id: &str,
		file_id: &str,
		total_fragments: u32,
		final_hash: &str,
	) -> Result<(), errors::UploadFinishFileError> {
		let mut jobs = self.jobs.blocking_lock();

		let job = jobs
			.iter_mut()
			.find(|us| us.id == job_id)
			.ok_or(errors::UploadFinishFileError::BadUploadJob)?;
		job.last_activity = Instant::now();

		let file = job
			.files
			.iter_mut()
			.find(|f| f.name == file_id)
			.ok_or(errors::UploadFinishFileError::BadFileID)?;
		if !matches!(file.state, UploadJobFileState::Pending) {
			return Err(errors::UploadFinishFileError::AlreadyFinished);
		}

		// This prevents the job from timing out if the actions
		// below take a long time
		file.state = UploadJobFileState::Finishing;

		let final_file_path = job.dir.join(file.name.as_str());
		let mut final_file = File::create(final_file_path)?;

		// We need these later, compute before locking
		let our_hash = {
			let mut hasher = Sha256::new();
			for frag_idx in 0..total_fragments {
				let frag_file_name = format!("{}-frag-{frag_idx}", file.name);
				let frag_hash = match file.frag_hashes.get(&frag_file_name) {
					Some(x) => x,
					None => {
						return Err(errors::UploadFinishFileError::MissingFragments {
							job_id: job_id.into(),
							file_id: file_id.into(),
							expected_fragments: total_fragments,
							missing_fragment: frag_idx,
						});
					}
				};
				hasher.update(frag_hash.as_bytes());
			}
			format!("{:X}", hasher.finalize())
		};

		// It would be nice to release the lock here,
		// but that doesn't seem to work---we get a deadlock when we
		// try to re-acquire the lock. why?
		// drop(jobs);

		for frag_idx in 0..total_fragments {
			let frag_file_name = format!("{}-frag-{frag_idx}", file.name);
			let frag_path = job.dir.join(&frag_file_name);

			if !frag_path.is_file() {
				return Err(errors::UploadFinishFileError::MissingFragments {
					job_id: job_id.into(),
					file_id: file_id.into(),
					expected_fragments: total_fragments,
					missing_fragment: frag_idx,
				});
			}

			let mut frag_file = File::open(&frag_path)?;
			std::io::copy(&mut frag_file, &mut final_file)?;
			std::fs::remove_file(frag_path)?;
		}

		job.last_activity = Instant::now();
		file.state = UploadJobFileState::Done;

		if our_hash != final_hash {
			warn!(
				message = "Uploaded hash does not match expected hash",
				job = ?job_id,
				file = ?file_id,
				expected_hash = ?final_hash,
				got_hash = ?our_hash
			);

			return Err(errors::UploadFinishFileError::HashDoesntMatch {
				actual: our_hash.into(),
				expected: final_hash.into(),
			});
		}
		info!(
			message = "Finished uploading file",
			job = ?job_id,
			file = ?file_id,
			hash = ?our_hash,
			file_type = ?file.mime,
		);

		return Ok(());
	}
}
