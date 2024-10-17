use copper_storaged::UserId;
use copper_util::{
	s3client::{MultipartUpload, S3Client},
	MimeType,
};
use errors::{NewUploadError, UploadAssignError, UploadFinishError, UploadFragmentError};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Display, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tracing::{debug, error, info};

use crate::config::EdgedConfig;

pub mod errors;

//
// MARK: Helpers
//

const UPLOAD_ID_LENGTH: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UploadJobId {
	id: SmartString<LazyCompact>,
}

impl UploadJobId {
	#[inline(always)]
	pub fn new() -> Self {
		let id: SmartString<LazyCompact> = rand::thread_rng()
			.sample_iter(&Alphanumeric)
			.take(UPLOAD_ID_LENGTH)
			.map(char::from)
			.collect();

		Self { id }
	}

	pub fn as_str(&self) -> &str {
		&self.id
	}
}

impl Display for UploadJobId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}

/// The state of an upload job
#[derive(Debug)]
enum UploadJobState {
	/// This job is pending, value is upload target
	Pending(MultipartUpload),

	/// This job is done, value is S3 object key.
	Done(SmartString<LazyCompact>),

	/// This job is done and has been assigned to a pipeline job.
	/// Value is the pipeline job's id.
	Assigned {
		key: SmartString<LazyCompact>,
		pipeline_job: SmartString<LazyCompact>,
	},
}

pub enum GotJobKey {
	NoSuchJob,
	JobNotDone,
	HereYouGo(SmartString<LazyCompact>),
	JobIsAssigned,
}

//
// MARK: UploadJob
//

pub struct UploadJob {
	pub id: UploadJobId,

	started_at: OffsetDateTime,
	last_activity: OffsetDateTime,
	state: UploadJobState,

	pub owner: UserId,
	pub mime: MimeType,
}

pub struct Uploader {
	config: Arc<EdgedConfig>,
	jobs: tokio::sync::Mutex<BTreeMap<UploadJobId, UploadJob>>,
	objectstore_client: Arc<S3Client>,
}

impl Uploader {
	/// Initialize a new upload manager
	pub fn new(config: Arc<EdgedConfig>, objectstore_client: Arc<S3Client>) -> Self {
		Self {
			config,
			jobs: tokio::sync::Mutex::new(BTreeMap::new()),
			objectstore_client,
		}
	}

	/// Get a finished upload job's object key.
	pub async fn get_job_object_key(&self, as_user: UserId, job_id: &UploadJobId) -> GotJobKey {
		let jobs = self.jobs.lock().await;

		let job = match jobs.get(job_id) {
			Some(x) => x,
			None => return GotJobKey::NoSuchJob,
		};

		// Make sure we are allowed to get this job
		if job.owner != as_user {
			return GotJobKey::NoSuchJob;
		}

		match &job.state {
			UploadJobState::Pending(_) => GotJobKey::JobNotDone,
			UploadJobState::Done(x) => GotJobKey::HereYouGo(x.clone()),
			UploadJobState::Assigned { .. } => GotJobKey::JobIsAssigned,
		}
	}

	/// Check all active jobs in this uploader,
	/// and remove those that that have timed out.
	///
	/// This should be called before any uploader action,
	/// which could result in a few old jobs laying around.
	/// This is not a problem, since they will never pile up
	/// and waste storage.
	#[inline(always)]
	async fn check_jobs(&self) {
		let mut jobs = self.jobs.lock().await;
		let now = OffsetDateTime::now_utc();
		let offset = Duration::from_secs(self.config.edged_upload_job_timeout);

		let mut to_remove = Vec::new();
		for (k, j) in jobs.iter() {
			let should_remove = match j.state {
				UploadJobState::Pending(_) => j.last_activity + offset < now,
				UploadJobState::Done(_) => j.last_activity + offset < now,

				// Assigned jobs never time out
				UploadJobState::Assigned { .. } => false,
			};

			if should_remove {
				debug!(
					message = "Job queued for removal",
					reason = "timeout",
					job_id = ?j.id,
					started_at = ?j.started_at,
					state = ?j.state
				);

				to_remove.push(k.clone());
				continue;
			}
		}

		for k in to_remove {
			debug!(message = "Removing job", reason = "timeout", job_id = ?k);
			let job = jobs.remove(&k).unwrap();
			match job.state {
				UploadJobState::Pending(uj) => uj.cancel().await,

				UploadJobState::Assigned { key, .. } | UploadJobState::Done(key) => {
					let res = self
						.objectstore_client
						.delete_object(&self.config.edged_objectstore_upload_bucket, &key)
						.await;
					match res {
						Ok(()) => {}
						Err(error) => {
							error!(message = "Could not delete uploaded object", ?key, ?error);
						}
					}
				}
			}
		}
	}
}

impl Uploader {
	/// Create a new upload job owned by the given user
	/// and return its id.
	pub async fn new_job(
		&self,
		owner: UserId,
		mime: MimeType,
	) -> Result<UploadJobId, NewUploadError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;

		// Generate a new id
		let id = loop {
			let id = UploadJobId::new();
			if !jobs.contains_key(&id) {
				break id;
			}
		};

		let now = OffsetDateTime::now_utc();
		jobs.insert(
			id.clone(),
			UploadJob {
				id: id.clone(),
				owner,
				started_at: now,
				last_activity: now,
				mime: mime.clone(),
				state: UploadJobState::Pending(
					self.objectstore_client
						.create_multipart_upload(
							&self.config.edged_objectstore_upload_bucket,
							format!("{}/{id}", i64::from(owner)).as_str(),
							mime,
						)
						.await?,
				),
			},
		);

		info!(
			message = "Created a new upload job",
			job_id = ?id,
		);

		return Ok(id.into());
	}

	/// Upload one fragment of an upload job.
	///
	/// Part numbers are consecutive and start at 1.
	/// If part number is none, we'll assume this is the "next" part.
	pub async fn upload_part(
		&self,
		as_user: UserId,
		job_id: &UploadJobId,
		data: &[u8],
		part_number: Option<i32>,
	) -> Result<(), UploadFragmentError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;
		let job = jobs.get_mut(job_id).ok_or(UploadFragmentError::BadUpload)?;

		// Cannot upload parts to a job that isn't pending
		if !matches!(job.state, UploadJobState::Pending(_)) {
			return Err(UploadFragmentError::BadUpload);
		}

		if job.owner != as_user {
			return Err(UploadFragmentError::NotMyUpload);
		}

		job.last_activity = OffsetDateTime::now_utc();
		let part_number = match part_number {
			Some(x) => x,
			None => match &mut job.state {
				UploadJobState::Pending(uj) => i32::try_from(uj.n_completed_parts()).unwrap() + 1,
				UploadJobState::Done(_) => unreachable!(),
				UploadJobState::Assigned { .. } => unreachable!(),
			},
		};

		assert!(
			part_number > 0,
			"Part numbers should be positive and start at 1"
		);

		// TODO: queue this future. CAREFUL WITH PART NUMBERS!
		match &mut job.state {
			UploadJobState::Pending(uj) => uj.upload_part(data, part_number).await?,
			UploadJobState::Done(_) => unreachable!(),
			UploadJobState::Assigned { .. } => unreachable!(),
		};

		return Ok(());
	}

	/// Finish an upload job as the given user
	pub async fn finish_job(
		&self,
		as_user: UserId,
		job_id: &UploadJobId,
	) -> Result<(), UploadFinishError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;
		let job = jobs.get_mut(job_id).ok_or(UploadFinishError::BadUpload)?;

		// Cannot finish a job that isn't pending
		if !matches!(job.state, UploadJobState::Pending(_)) {
			return Err(UploadFinishError::BadUpload);
		}

		if job.owner != as_user {
			return Err(UploadFinishError::NotMyUpload);
		}

		let done_state = UploadJobState::Done(match &job.state {
			UploadJobState::Pending(uj) => uj.key().into(),
			UploadJobState::Done(_) => unreachable!(),
			UploadJobState::Assigned { .. } => unreachable!(),
		});

		let uj = std::mem::replace(&mut job.state, done_state);

		match uj {
			UploadJobState::Pending(uj) => uj.finish().await?,
			UploadJobState::Done(_) => unreachable!(),
			UploadJobState::Assigned { .. } => unreachable!(),
		};

		debug!(
			message = "Finished upload",
			job_id = ?job_id,
			mime = ?job.mime,
		);

		return Ok(());
	}

	/// Finish an upload job as the given user
	pub async fn assign_job_to_pipeline(
		&self,
		as_user: UserId,
		job_id: &UploadJobId,
		to_pipeline_job: &str,
	) -> Result<(), UploadAssignError> {
		self.check_jobs().await;

		let mut jobs = self.jobs.lock().await;
		let job = jobs.get_mut(job_id).ok_or(UploadAssignError::BadUpload)?;

		// Cannot assign a job that isn't done
		let key = match &job.state {
			UploadJobState::Done(x) => x.clone(),
			_ => return Err(UploadAssignError::BadUpload),
		};

		if job.owner != as_user {
			return Err(UploadAssignError::NotMyUpload);
		}

		let _ = std::mem::replace(
			&mut job.state,
			UploadJobState::Assigned {
				key,
				pipeline_job: to_pipeline_job.into(),
			},
		);

		debug!(
			message = "Assigned upload job",
			job_id = ?job_id,
			mime = ?job.mime,
			to_pipeline_job
		);

		return Ok(());
	}
}
