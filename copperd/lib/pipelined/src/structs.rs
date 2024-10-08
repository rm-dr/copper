use copper_storaged::UserId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JobCounts {
	pub total_jobs: usize,
	pub queued_jobs: usize,
	pub running_jobs: usize,
	pub successful_jobs: usize,
	pub failed_jobs: usize,
	pub build_errors: usize,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JobInfo {
	pub id: String,

	#[schema(value_type = i64)]
	pub owner: UserId,

	pub state: JobInfoState,

	#[schema(value_type = String)]
	pub added_at: OffsetDateTime,

	#[schema(value_type = Option<String>)]
	pub started_at: Option<OffsetDateTime>,

	#[schema(value_type = Option<String>)]
	pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum JobInfoState {
	Queued,
	Running,
	Success,
	Failed,
	BuildError { message: String },
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JobInfoList {
	pub counts: JobCounts,

	/// The number of jobs we skipped while paginating.
	/// (i.e, the true index of the first job in `jobs`)
	pub skip: usize,
	pub jobs: Vec<JobInfo>,
}
