use async_trait::async_trait;
use copper_edged::{PipelineId, PipelineInfo, UserInfo, UserPassword};
use copper_pipelined::json::PipelineJson;
use copper_storaged::UserId;
use copper_util::names::check_name;
use sqlx::{Connection, Row};

use super::PgDatabaseClient;
use crate::database::base::{
	client::DatabaseClient,
	errors::{
		pipeline::{
			AddPipelineError, DeletePipelineError, GetPipelineError, ListPipelineError,
			UpdatePipelineError,
		},
		user::{AddUserError, DeleteUserError, GetUserError, UpdateUserError},
	},
};

#[async_trait]
impl DatabaseClient for PgDatabaseClient {
	//
	// MARK: User
	//

	async fn add_user(
		&self,
		email: &str,
		name: &str,
		password: &UserPassword,
	) -> Result<UserId, AddUserError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddUserError::NameError(e)),
		}

		// Start transaction
		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		let res = sqlx::query(
			"
			INSERT INTO users (user_email, user_name, user_pass)
			VALUES ($1, $2, $3)
			RETURNING id;
			",
		)
		.bind(email)
		.bind(name)
		.bind(serde_json::to_string(password).unwrap())
		.fetch_one(&mut *t)
		.await;

		t.commit().await?;

		let new_user: UserId = match res {
			Ok(row) => row.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddUserError::UniqueEmailViolation);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(AddUserError::DbError(e)),
		};

		return Ok(new_user);
	}

	async fn get_user(&self, user: UserId) -> Result<Option<UserInfo>, GetUserError> {
		let mut conn = self.pool.acquire().await?;

		let res = sqlx::query("SELECT * FROM users WHERE id=$1;")
			.bind(i64::from(user))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(Some(UserInfo {
				id: res.get::<i64, _>("id").into(),
				name: res.get::<String, _>("user_name").into(),
				email: res.get::<String, _>("user_email").into(),
				password: serde_json::from_str(res.get::<&str, _>("user_pass")).unwrap(),
			})),
		};
	}

	async fn get_user_by_email(&self, email: &str) -> Result<Option<UserInfo>, GetUserError> {
		let mut conn = self.pool.acquire().await?;

		let res = sqlx::query("SELECT * FROM users WHERE user_email=$1;")
			.bind(email)
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(Some(UserInfo {
				id: res.get::<i64, _>("id").into(),
				name: res.get::<String, _>("user_name").into(),
				email: res.get::<String, _>("user_email").into(),
				password: serde_json::from_str(res.get::<&str, _>("user_pass")).unwrap(),
			})),
		};
	}

	async fn update_user(&self, new_info: &UserInfo) -> Result<(), UpdateUserError> {
		match check_name(&new_info.name) {
			Ok(()) => {}
			Err(e) => return Err(UpdateUserError::NameError(e)),
		}

		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		let res =
			sqlx::query("UPDATE users SET user_name=$1, user_email=$2, user_pass=$3 WHERE id=$4;")
				.bind(new_info.name.as_str())
				.bind(new_info.email.as_str())
				.bind(serde_json::to_string(&new_info.password).unwrap())
				.bind(i64::from(new_info.id))
				.execute(&mut *t)
				.await;

		t.commit().await?;

		return match res {
			Ok(_) => Ok(()),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(UpdateUserError::UniqueEmailViolation)
				} else {
					Err(sqlx::Error::Database(e).into())
				}
			}
			Err(e) => Err(e.into()),
		};
	}

	async fn del_user(&self, user: UserId) -> Result<(), DeleteUserError> {
		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		// TODO: we still need to delete this user's data,
		// since it's stored in a different db.
		sqlx::query("DELETE FROM users WHERE id=$1;")
			.bind(i64::from(user))
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		return Ok(());
	}

	//
	// MARK: Pipeline
	//

	async fn add_pipeline(
		&self,
		for_user: UserId,
		name: &str,
		pipeline: &PipelineJson,
	) -> Result<PipelineInfo, AddPipelineError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddPipelineError::NameError(e)),
		}

		// Start transaction
		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		let res = sqlx::query(
			"
			INSERT INTO pipelines (owned_by, name, data)
			VALUES ($1, $2, $3)
			RETURNING id;
			",
		)
		.bind(i64::from(for_user))
		.bind(name)
		.bind(serde_json::to_string(pipeline).unwrap())
		.fetch_one(&mut *t)
		.await;

		t.commit().await?;

		let new_pipeline_id: PipelineId = match res {
			Ok(row) => row.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddPipelineError::UniqueViolation);
				} else {
					return Err(sqlx::Error::Database(e).into());
				}
			}
			Err(e) => return Err(e.into()),
		};

		return Ok(PipelineInfo {
			id: new_pipeline_id,
			owned_by: for_user,
			name: name.into(),
			data: pipeline.clone(),
		});
	}

	async fn list_pipelines(
		&self,
		for_user: UserId,
	) -> Result<Vec<PipelineInfo>, ListPipelineError> {
		let mut conn = self.pool.acquire().await?;

		let res = sqlx::query("SELECT * FROM pipelines WHERE owned_by=$1;")
			.bind(i64::from(for_user))
			.fetch_all(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(Vec::new()),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(res
				.into_iter()
				.map(|row| PipelineInfo {
					id: row.get::<i64, _>("id").into(),
					owned_by: row.get::<i64, _>("owned_by").into(),
					name: row.get::<String, _>("name").into(),
					data: serde_json::from_str(row.get::<&str, _>("data")).unwrap(),
				})
				.collect()),
		};
	}

	async fn get_pipeline(
		&self,
		pipeline: PipelineId,
	) -> Result<Option<PipelineInfo>, GetPipelineError> {
		// TODO: handle deserialize failure

		let mut conn = self.pool.acquire().await?;

		let res = sqlx::query("SELECT * FROM pipelines WHERE id=$1;")
			.bind(i64::from(pipeline))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(e.into()),
			Ok(res) => Ok(Some(PipelineInfo {
				id: res.get::<i64, _>("id").into(),
				owned_by: res.get::<i64, _>("owned_by").into(),
				name: res.get::<String, _>("name").into(),
				data: serde_json::from_str(res.get::<&str, _>("data")).unwrap(),
			})),
		};
	}

	async fn update_pipeline(
		&self,
		new_info: &PipelineInfo,
	) -> Result<PipelineInfo, UpdatePipelineError> {
		match check_name(&new_info.name) {
			Ok(()) => {}
			Err(e) => return Err(UpdatePipelineError::NameError(e)),
		}

		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		let res = sqlx::query("UPDATE pipelines SET owned_by=$1, name=$2, data=$3 WHERE id=$4;")
			.bind(i64::from(new_info.owned_by))
			.bind(new_info.name.as_str())
			.bind(serde_json::to_string(&new_info.data).unwrap())
			.bind(i64::from(new_info.id))
			.execute(&mut *t)
			.await;

		t.commit().await?;

		return match res {
			Ok(_) => Ok(new_info.clone()),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(UpdatePipelineError::UniqueViolation)
				} else {
					Err(sqlx::Error::Database(e).into())
				}
			}
			Err(e) => Err(e.into()),
		};
	}

	async fn del_pipeline(&self, pipeline: PipelineId) -> Result<(), DeletePipelineError> {
		let mut conn = self.pool.acquire().await?;
		let mut t = conn.begin().await?;

		// TODO: we still need to delete this user's data,
		// since it's stored in a different db.
		sqlx::query("DELETE FROM pipelines WHERE id=$1;")
			.bind(i64::from(pipeline))
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		return Ok(());
	}
}
