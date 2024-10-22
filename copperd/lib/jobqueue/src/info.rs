//! Helper structs that contain database element properties

use std::collections::BTreeMap;

use copper_pipelined::json::PipelineJson;
use copper_storaged::{AttrData, UserId};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::id::QueuedJobId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "state")]
pub enum QueuedJobState {
	Queued,
	Running,
	Success,
	Failed,
	BuildError { message: String },
}

#[derive(Serialize, Deserialize)]
pub struct QueuedJobInfo {
	/// A unique id for this job
	pub job_id: QueuedJobId,

	/// The user that owns this job
	pub owned_by: UserId,

	/// The state of this job
	pub state: QueuedJobState,

	/// The pipeline to run
	pub pipeline: PipelineJson,

	/// When this job was created
	pub created_at: OffsetDateTime,

	/// When this job was started
	pub started_at: Option<OffsetDateTime>,

	/// When this job was finished
	pub finished_at: Option<OffsetDateTime>,

	/// The input to pass to this pipeline
	pub input: BTreeMap<SmartString<LazyCompact>, AttrData>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QueuedJobInfoShort {
	/// A unique id for this job
	#[schema(value_type = String)]
	pub job_id: QueuedJobId,

	/// The user that owns this job
	#[schema(value_type = i64)]
	pub owned_by: UserId,

	/// The state of this job
	pub state: QueuedJobState,

	#[schema(value_type = String)]
	pub created_at: OffsetDateTime,

	#[schema(value_type = Option<String>)]
	pub started_at: Option<OffsetDateTime>,

	#[schema(value_type = Option<String>)]
	pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QueuedJobCounts {
	pub total_jobs: i64,
	pub queued_jobs: i64,
	pub running_jobs: i64,
	pub successful_jobs: i64,
	pub failed_jobs: i64,
	pub build_errors: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct QueuedJobInfoList {
	pub counts: QueuedJobCounts,

	/// The number of jobs we skipped while paginating.
	/// (i.e, the true index of the first job in `jobs`)
	pub skip: i64,
	pub jobs: Vec<QueuedJobInfoShort>,
}
