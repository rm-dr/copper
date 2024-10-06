use copper_pipelined::helpers::{MultipartUpload, S3Client};
use copper_storaged::UserId;
use copper_util::MimeType;
use errors::{NewUploadError, UploadFinishError, UploadFragmentError};
use rand::{distributions::Alphanumeric, Rng};
use smartstring::{LazyCompact, SmartString};
use std::{sync::Arc, time::Duration};
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::config::EdgedConfig;

pub mod errors;

const UPLOAD_ID_LENGTH: usize = 16;

pub struct UploadJob {
	pub id: SmartString<LazyCompact>,
	pub started_at: OffsetDateTime,
	pub last_activity: OffsetDateTime,
	pub owner: UserId,

	pub uploadjob: MultipartUpload,
	pub mime: MimeType,
}

pub struct Uploader {
	config: Arc<EdgedConfig>,
	jobs: tokio::sync::Mutex<Vec<UploadJob>>,
	objectstore_client: Arc<S3Client>,
}

impl Uploader {
	pub fn new(config: Arc<EdgedConfig>, objectstore_client: Arc<S3Client>) -> Self {
		Self {
			config,
			jobs: tokio::sync::Mutex::new(Vec::new()),
			objectstore_client,
		}
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
	/// and remove those we no longer need.
	///
	/// This cleans up jobs that have timed out,
	/// and jobs bound to a pipeline that has been finished.
	#[inline(always)]
	async fn check_jobs(&self) {
		let mut jobs = self.jobs.lock().await;
		let now = OffsetDateTime::now_utc();
		let offset = Duration::from_secs(self.config.edged_upload_job_timeout);

		let mut i = 0;
		while i < jobs.len() {
			let j = &jobs[i];

			if j.last_activity + offset < now {
				debug!(
					message = "Removing job",
					reason = "timeout",
					job_id = ?j.id,
					started_at = ?j.started_at
				);

				let job = jobs.swap_remove(i);
				job.uploadjob.cancel().await;

				continue;
			}

			i += 1;
		}
	}
}

impl Uploader {
	pub async fn new_job(
		&self,
		owner: UserId,
		mime: MimeType,
	) -> Result<SmartString<LazyCompact>, NewUploadError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;
		let id = loop {
			let id = Uploader::generate_id();
			// TODO: check existing S3 objects
			if jobs.iter().all(|us| us.id != id) {
				break id;
			}
		};

		let now = OffsetDateTime::now_utc();
		jobs.push(UploadJob {
			id: id.clone(),
			owner,
			started_at: now,
			last_activity: now,
			mime: mime.clone(),
			uploadjob: self
				.objectstore_client
				.create_multipart_upload(&id, mime)
				.await?,
		});

		info!(
			message = "Created a new upload job",
			job_id = ?id,
		);

		return Ok(id);
	}

	/// Upload one fragment of an upload job.
	///
	/// Part numbers are consecutive and start at 1.
	/// If part number is none, we'll assume this is the "next" part.
	pub async fn upload_part(
		&self,
		as_user: UserId,
		job_id: &str,
		data: &[u8],
		part_number: Option<i32>,
	) -> Result<(), UploadFragmentError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;
		let job = jobs
			.iter_mut()
			.find(|us| us.id == job_id)
			.ok_or(UploadFragmentError::BadUpload)?;
		if job.owner != as_user {
			return Err(UploadFragmentError::NotMyUpload);
		}

		job.last_activity = OffsetDateTime::now_utc();
		let part_number = match part_number {
			Some(x) => x,
			None => i32::try_from(job.uploadjob.n_completed_parts()).unwrap() + 1,
		};

		assert!(
			part_number > 0,
			"Part numbers should be positive and start at 1"
		);

		// TODO: queue this future. CAREFUL WITH PART NUMBERS!
		job.uploadjob.upload_part(data, part_number).await?;

		return Ok(());
	}

	pub async fn finish_job(&self, as_user: UserId, job_id: &str) -> Result<(), UploadFinishError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;
		let job_idx = jobs
			.iter_mut()
			.enumerate()
			.find(|(_, us)| us.id == job_id)
			.ok_or(UploadFinishError::BadUpload)?;
		if job_idx.1.owner != as_user {
			return Err(UploadFinishError::NotMyUpload);
		}

		let job_idx = job_idx.0;
		let job = jobs.swap_remove(job_idx);

		job.uploadjob.finish().await?;

		debug!(
			message = "Finished upload",
			job_id = ?job_id,
			mime = ?job.mime,
		);

		return Ok(());
	}
}
