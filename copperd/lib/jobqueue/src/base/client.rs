//! The job queue client api

use async_trait::async_trait;
use copper_piper::json::PipelineJson;
use copper_itemdb::{AttrData, UserId};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use crate::{
	id::QueuedJobId,
	info::{QueuedJobInfo, QueuedJobInfoList, QueuedJobInfoShort},
};

use super::errors::{
	AddJobError, BuildErrorJobError, FailJobError, GetJobShortError, GetQueuedJobError,
	GetUserJobsError, SuccessJobError,
};

/// A generic job queue
#[async_trait]
pub trait JobQueueClient
where
	Self: Send + Sync,
{
	/// Queue a new job
	async fn add_job(
		&self,
		job_id: QueuedJobId,
		owned_by: UserId,
		pipeline: &PipelineJson,
		input: &BTreeMap<SmartString<LazyCompact>, AttrData>,
	) -> Result<QueuedJobId, AddJobError>;

	/// Get a job by id
	async fn get_job_short(
		&self,
		job_id: &QueuedJobId,
	) -> Result<QueuedJobInfoShort, GetJobShortError>;

	/// List all a user's jobs
	async fn get_user_jobs(
		&self,
		owned_by: UserId,
		skip: i64,
		count: i64,
	) -> Result<QueuedJobInfoList, GetUserJobsError>;

	/// Get the oldest job with `state = Queued` and set `state = Running`
	/// The returned QueuedJobInfo should have `state = Running`.
	///
	/// This action must be globally atomic. Only one process should
	/// ever get a queued job.
	async fn get_queued_job(&self) -> Result<Option<QueuedJobInfo>, GetQueuedJobError>;

	/// Atomically mark the given job as `BuildError`.
	/// If this job is not `Running`, throw an error.
	async fn builderror_job(
		&self,
		job_id: &QueuedJobId,
		error_message: &str,
	) -> Result<(), BuildErrorJobError>;

	/// Atomically mark the given job as `Failed`.
	/// If this job is not `Running`, throw an error.
	async fn fail_job(&self, job_id: &QueuedJobId) -> Result<(), FailJobError>;

	/// Atomically mark the given job as `Success`.
	/// If this job is not `Running`, throw an error.
	async fn success_job(&self, job_id: &QueuedJobId) -> Result<(), SuccessJobError>;
}
