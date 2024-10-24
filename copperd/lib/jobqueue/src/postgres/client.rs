use std::collections::BTreeMap;

use async_trait::async_trait;
use copper_itemdb::{AttrData, UserId};
use copper_piper::json::PipelineJson;
use smartstring::{LazyCompact, SmartString};
use sqlx::{
	types::{time::OffsetDateTime, Json},
	Connection, Row,
};

use super::PgJobQueueClient;
use crate::{
	base::{
		client::JobQueueClient,
		errors::{
			AddJobError, BuildErrorJobError, FailJobError, GetJobShortError, GetQueuedJobError,
			GetUserJobsError, SuccessJobError,
		},
	},
	id::QueuedJobId,
	info::{QueuedJobCounts, QueuedJobInfo, QueuedJobInfoList, QueuedJobInfoShort, QueuedJobState},
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
				QueuedJobState::Queued => counts.queued_jobs += n,
				QueuedJobState::Running => counts.running_jobs += n,
				QueuedJobState::Success { .. } => counts.successful_jobs += n,
				QueuedJobState::FailedRunning { .. } => counts.failed_jobs += n,
				QueuedJobState::FailedTransaction { .. } => counts.failed_jobs += n,
				QueuedJobState::BuildError { .. } => counts.build_errors += n,
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

	async fn get_queued_job(&self) -> Result<Option<QueuedJobInfo>, GetQueuedJobError> {
		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query(
			"
			UPDATE jobs
			SET state = $1, started_at = $2
			WHERE id = (
				SELECT id
				FROM jobs
				WHERE state = $3
				ORDER BY created_at ASC
				FOR UPDATE SKIP LOCKED
				LIMIT 1
			)
			RETURNING *;
			",
		)
		.bind(serde_json::to_string(&QueuedJobState::Running).unwrap())
		.bind(OffsetDateTime::now_utc())
		.bind(serde_json::to_string(&QueuedJobState::Queued).unwrap())
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(Some(QueuedJobInfo {
				job_id: res.get::<&str, _>("id").into(),
				owned_by: res.get::<i64, _>("owned_by").into(),
				created_at: res.get("created_at"),
				started_at: res.get("started_at"),
				finished_at: res.get("finished_at"),
				state: serde_json::from_str(res.get::<&str, _>("state").into()).unwrap(),
				pipeline: res.get::<sqlx::types::Json<PipelineJson>, _>("pipeline").0,
				input: res
					.get::<sqlx::types::Json<BTreeMap<SmartString<LazyCompact>, AttrData>>, _>(
						"input",
					)
					.0,
			})),
		};
	}

	async fn builderror_job(
		&self,
		job_id: &QueuedJobId,
		error_message: &str,
	) -> Result<(), BuildErrorJobError> {
		let mut conn = self.pool.acquire().await?;
		// RETURNING id is required, RowNotFound is always thrown if it is removed.
		let res = sqlx::query(
			"
			UPDATE jobs
			SET state = $1, finished_at = $2
			WHERE id = $3
			AND state = $4
			RETURNING id;
			",
		)
		.bind(
			serde_json::to_string(&QueuedJobState::BuildError {
				message: error_message.into(),
			})
			.unwrap(),
		)
		.bind(OffsetDateTime::now_utc())
		.bind(job_id.as_str())
		.bind(serde_json::to_string(&QueuedJobState::Running).unwrap())
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(BuildErrorJobError::NotRunning),
			Err(e) => Err(e.into()),
			Ok(_) => Ok(()),
		};
	}

	async fn fail_job_run(&self, job_id: &QueuedJobId, message: &str) -> Result<(), FailJobError> {
		let mut conn = self.pool.acquire().await?;
		// RETURNING id is required, RowNotFound is always thrown if it is removed.
		let res = sqlx::query(
			"
			UPDATE jobs
			SET state = $1, finished_at = $2
			WHERE id = $3
			AND state = $4
			RETURNING id;
			",
		)
		.bind(
			serde_json::to_string(&QueuedJobState::FailedRunning {
				message: message.into(),
			})
			.unwrap(),
		)
		.bind(OffsetDateTime::now_utc())
		.bind(job_id.as_str())
		.bind(serde_json::to_string(&QueuedJobState::Running).unwrap())
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(FailJobError::NotRunning),
			Err(e) => Err(e.into()),
			Ok(_) => Ok(()),
		};
	}

	async fn fail_job_transaction(
		&self,
		job_id: &QueuedJobId,
		message: &str,
	) -> Result<(), FailJobError> {
		let mut conn = self.pool.acquire().await?;
		// RETURNING id is required, RowNotFound is always thrown if it is removed.
		let res = sqlx::query(
			"
			UPDATE jobs
			SET state = $1, finished_at = $2
			WHERE id = $3
			AND state = $4
			RETURNING id;
			",
		)
		.bind(
			serde_json::to_string(&QueuedJobState::FailedTransaction {
				message: message.into(),
			})
			.unwrap(),
		)
		.bind(OffsetDateTime::now_utc())
		.bind(job_id.as_str())
		.bind(serde_json::to_string(&QueuedJobState::Running).unwrap())
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(FailJobError::NotRunning),
			Err(e) => Err(e.into()),
			Ok(_) => Ok(()),
		};
	}

	async fn success_job(&self, job_id: &QueuedJobId) -> Result<(), SuccessJobError> {
		let mut conn = self.pool.acquire().await?;
		// RETURNING id is required, RowNotFound is always thrown if it is removed.
		let res = sqlx::query(
			"
			UPDATE jobs
			SET state = $1, finished_at = $2
			WHERE id = $3
			AND state = $4
			RETURNING id;
			",
		)
		.bind(serde_json::to_string(&QueuedJobState::Success).unwrap())
		.bind(OffsetDateTime::now_utc())
		.bind(job_id.as_str())
		.bind(serde_json::to_string(&QueuedJobState::Running).unwrap())
		.fetch_one(&mut *conn)
		.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(SuccessJobError::NotRunning),
			Err(e) => Err(e.into()),
			Ok(_) => Ok(()),
		};
	}
}
