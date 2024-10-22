use std::collections::BTreeMap;

use async_trait::async_trait;
use copper_pipelined::json::PipelineJson;
use copper_storaged::{AttrData, UserId};
use smartstring::{LazyCompact, SmartString};
use sqlx::{
	types::{time::OffsetDateTime, Json},
	Connection, Row,
};

use super::PgJobQueueClient;
use crate::{
	base::{
		client::JobQueueClient,
		errors::{AddJobError, GetJobShortError, GetUserJobsError},
	},
	id::QueuedJobId,
	info::{QueuedJobCounts, QueuedJobInfoList, QueuedJobInfoShort, QueuedJobState},
};

#[async_trait]
impl JobQueueClient for PgJobQueueClient {
	/// Queue a new job
	async fn add_job(
		&self,
		job_id: QueuedJobId,
		owned_by: UserId,
		pipeline: &PipelineJson,
		input: &BTreeMap<SmartString<LazyCompact>, AttrData>,
	) -> Result<QueuedJobId, AddJobError> {
		// Start transaction
		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		let res = sqlx::query(
			"
			INSERT INTO jobs (id, created_at, owned_by, state, pipeline, input)
			VALUES ($1, $2, $3, $4, $5, $6)
			RETURNING id;
			",
		)
		.bind(job_id.as_str())
		.bind(OffsetDateTime::now_utc())
		.bind(i64::from(owned_by))
		.bind(serde_json::to_string(&QueuedJobState::Queued).unwrap())
		.bind(Json::from(pipeline))
		.bind(Json::from(input))
		.fetch_one(&mut *t)
		.await;

		t.commit().await?;

		let new_job_id: QueuedJobId = match res {
			Ok(row) => row.get::<&str, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddJobError::AlreadyExists);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(AddJobError::DbError(e)),
		};

		return Ok(new_job_id);
	}

	/// Get a job by id
	async fn get_job_short(
		&self,
		job_id: &QueuedJobId,
	) -> Result<QueuedJobInfoShort, GetJobShortError> {
		let mut conn = self.pool.acquire().await?;

		let res = sqlx::query(
			"
			SELECT id, owned_by, created_at, started_at, finished_at, state
			FROM jobs
			WHERE id=$1
			",
		)
		.bind(job_id.as_str())
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetJobShortError::NotFound),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(QueuedJobInfoShort {
				job_id: res.get::<&str, _>("id").into(),
				owned_by: res.get::<i64, _>("owned_by").into(),
				created_at: res.get("created_at"),
				started_at: res.get("started_at"),
				finished_at: res.get("finished_at"),
				state: serde_json::from_str(res.get::<&str, _>("state").into()).unwrap(),
			}),
		};
	}

	async fn get_user_jobs(
		&self,
		owned_by: UserId,
		skip: i64,
		count: i64,
	) -> Result<QueuedJobInfoList, GetUserJobsError> {
		let mut conn = self.pool.acquire().await?;

		let res = sqlx::query(
			"
			SELECT id, owned_by, created_at, started_at, finished_at, state
			FROM jobs
			WHERE owned_by=$1
			ORDER BY started_at DESC
			OFFSET $2
			LIMIT $3
			",
		)
		.bind(i64::from(owned_by))
		.bind(skip)
		.bind(count)
		.fetch_all(&mut *conn)
		.await?;

		let mut out = Vec::new();
		for row in res {
			out.push(QueuedJobInfoShort {
				job_id: row.get::<&str, _>("id").into(),
				owned_by: row.get::<i64, _>("owned_by").into(),
				created_at: row.get("created_at"),
				started_at: row.get("started_at"),
				finished_at: row.get("finished_at"),
				state: serde_json::from_str(row.get::<&str, _>("state").into()).unwrap(),
			})
		}

		let mut counts = QueuedJobCounts {
			queued_jobs: 0,
			running_jobs: 0,
			successful_jobs: 0,
			failed_jobs: 0,
			build_errors: 0,

			total_jobs: 0,
		};

		let res = sqlx::query(
			"
			SELECT state, COUNT(*)
			FROM jobs
			WHERE owned_by=$1
			GROUP BY state;
			",
		)
		.bind(i64::from(owned_by))
		.fetch_all(&mut *conn)
		.await?;

		for row in res {
			let state: QueuedJobState =
				serde_json::from_str(row.get::<&str, _>("state").into()).unwrap();
			let n: i64 = row.get("count");

			match state {
				QueuedJobState::Queued => counts.queued_jobs = n,
				QueuedJobState::Running => counts.running_jobs = n,
				QueuedJobState::Success => counts.successful_jobs = n,
				QueuedJobState::Failed => counts.failed_jobs = n,
				QueuedJobState::BuildError { .. } => counts.build_errors = n,
			}
		}

		counts.total_jobs = counts.queued_jobs
			+ counts.running_jobs
			+ counts.successful_jobs
			+ counts.failed_jobs
			+ counts.build_errors;

		return Ok(QueuedJobInfoList {
			skip,
			counts,
			jobs: out,
		});
	}
}
