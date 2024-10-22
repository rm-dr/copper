//! The job queue client api

use async_trait::async_trait;
use copper_pipelined::json::PipelineJson;
use copper_storaged::{AttrData, UserId};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use crate::{
	id::QueuedJobId,
	info::{QueuedJobInfoList, QueuedJobInfoShort},
};

use super::errors::{AddJobError, GetJobShortError, GetUserJobsError};

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
}
