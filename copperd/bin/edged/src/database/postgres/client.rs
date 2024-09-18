use async_trait::async_trait;
use copper_edged::{PipelineId, PipelineInfo, UserId, UserInfo, UserPassword};
use copper_pipelined::json::PipelineJson;
use copper_util::{names::check_name, MimeType};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};

use super::PgDatabaseClient;
use crate::database::base::{
	client::DatabaseClient,
	errors::{
		pipeline::{AddPipelineError, DeletePipelineError, GetPipelineError, UpdatePipelineError},
		user::{AddUserError, DeleteUserError, GetUserError, UpdateUserError},
	},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct BlobJsonEncoded {
	url: String,
	mime: MimeType,
}

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
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddUserError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddUserError::DbError(Box::new(e)))?;

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

		t.commit()
			.await
			.map_err(|e| AddUserError::DbError(Box::new(e)))?;

		let new_user: UserId = match res {
			Ok(row) => row.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddUserError::UniqueEmailViolation);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddUserError::DbError(e));
				}
			}
			Err(e) => return Err(AddUserError::DbError(Box::new(e))),
		};

		return Ok(new_user);
	}

	async fn get_user(&self, user: UserId) -> Result<Option<UserInfo>, GetUserError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetUserError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM users WHERE id=$1;")
			.bind(i64::from(user))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(GetUserError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(UserInfo {
				id: res.get::<i64, _>("id").into(),
				name: res.get::<String, _>("user_name").into(),
				email: res.get::<String, _>("user_email").into(),
				password: serde_json::from_str(res.get::<&str, _>("user_pass")).unwrap(),
			})),
		};
	}

	async fn get_user_by_email(&self, email: &str) -> Result<Option<UserInfo>, GetUserError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetUserError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM users WHERE user_email=$1;")
			.bind(email)
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => Err(GetUserError::DbError(Box::new(e))),
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

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| UpdateUserError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| UpdateUserError::DbError(Box::new(e)))?;

		let res =
			sqlx::query("UPDATE users SET user_name=$1, user_email=$2, user_pass=$3 WHERE id=$4;")
				.bind(new_info.name.as_str())
				.bind(new_info.email.as_str())
				.bind(serde_json::to_string(&new_info.password).unwrap())
				.bind(i64::from(new_info.id))
				.execute(&mut *t)
				.await;

		t.commit()
			.await
			.map_err(|e| UpdateUserError::DbError(Box::new(e)))?;

		return match res {
			Ok(_) => Ok(()),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(UpdateUserError::UniqueEmailViolation)
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					Err(UpdateUserError::DbError(e))
				}
			}
			Err(e) => Err(UpdateUserError::DbError(Box::new(e))),
		};
	}

	async fn del_user(&self, user: UserId) -> Result<(), DeleteUserError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeleteUserError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| DeleteUserError::DbError(Box::new(e)))?;

		// TODO: we still need to delete this user's data,
		// since it's stored in a different db.
		sqlx::query("DELETE FROM users WHERE id=$1;")
			.bind(i64::from(user))
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteUserError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeleteUserError::DbError(Box::new(e)))?;

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
	) -> Result<PipelineId, AddPipelineError> {
		match check_name(name) {
			Ok(()) => {}
			Err(e) => return Err(AddPipelineError::NameError(e)),
		}

		// Start transaction
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| AddPipelineError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| AddPipelineError::DbError(Box::new(e)))?;

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

		t.commit()
			.await
			.map_err(|e| AddPipelineError::DbError(Box::new(e)))?;

		let new_pipeline: PipelineId = match res {
			Ok(row) => row.get::<i64, _>("id").into(),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					return Err(AddPipelineError::UniqueViolation);
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					return Err(AddPipelineError::DbError(e));
				}
			}
			Err(e) => return Err(AddPipelineError::DbError(Box::new(e))),
		};

		return Ok(new_pipeline);
	}

	async fn get_pipeline(
		&self,
		pipeline: PipelineId,
	) -> Result<Option<PipelineInfo>, GetPipelineError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetPipelineError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM pipelines WHERE id=$1;")
			.bind(i64::from(pipeline))
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(GetPipelineError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(PipelineInfo {
				id: res.get::<i64, _>("id").into(),
				owned_by: res.get::<i64, _>("owned_by").into(),
				name: res.get::<String, _>("name").into(),
				data: serde_json::from_str(res.get::<&str, _>("data")).unwrap(),
			})),
		};
	}

	async fn get_pipeline_by_name(
		&self,
		user: UserId,
		pipeline_name: &str,
	) -> Result<Option<PipelineInfo>, GetPipelineError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetPipelineError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM pipelines WHERE owned_by=$1 AND name=$2;")
			.bind(i64::from(user))
			.bind(pipeline_name)
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(GetPipelineError::DbError(Box::new(e))),
			Ok(res) => Ok(Some(PipelineInfo {
				id: res.get::<i64, _>("id").into(),
				owned_by: res.get::<i64, _>("owned_by").into(),
				name: res.get::<String, _>("name").into(),
				data: serde_json::from_str(res.get::<&str, _>("data")).unwrap(),
			})),
		};
	}

	async fn update_pipeline(&self, new_info: &PipelineInfo) -> Result<(), UpdatePipelineError> {
		match check_name(&new_info.name) {
			Ok(()) => {}
			Err(e) => return Err(UpdatePipelineError::NameError(e)),
		}

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| UpdatePipelineError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| UpdatePipelineError::DbError(Box::new(e)))?;

		let res = sqlx::query("UPDATE pipelines SET owned_by=$1, name=$2, data=$3 WHERE id=$4;")
			.bind(i64::from(new_info.owned_by))
			.bind(new_info.name.as_str())
			.bind(serde_json::to_string(&new_info.data).unwrap())
			.bind(i64::from(new_info.id))
			.execute(&mut *t)
			.await;

		t.commit()
			.await
			.map_err(|e| UpdatePipelineError::DbError(Box::new(e)))?;

		return match res {
			Ok(_) => Ok(()),
			Err(sqlx::Error::Database(e)) => {
				if e.is_unique_violation() {
					Err(UpdatePipelineError::UniqueViolation)
				} else {
					let e = Box::new(sqlx::Error::Database(e));
					Err(UpdatePipelineError::DbError(e))
				}
			}
			Err(e) => Err(UpdatePipelineError::DbError(Box::new(e))),
		};
	}

	async fn del_pipeline(&self, pipeline: PipelineId) -> Result<(), DeletePipelineError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeletePipelineError::DbError(Box::new(e)))?;
		let mut t = conn
			.begin()
			.await
			.map_err(|e| DeletePipelineError::DbError(Box::new(e)))?;

		// TODO: we still need to delete this user's data,
		// since it's stored in a different db.
		sqlx::query("DELETE FROM pipelines WHERE id=$1;")
			.bind(i64::from(pipeline))
			.execute(&mut *t)
			.await
			.map_err(|e| DeletePipelineError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeletePipelineError::DbError(Box::new(e)))?;

		return Ok(());
	}
}
