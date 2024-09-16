use async_trait::async_trait;
use copper_edged::{UserId, UserInfo, UserPassword};
use copper_util::{names::check_name, MimeType};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Row};

use super::PgDatabaseClient;
use crate::database::base::{
	client::DatabaseClient,
	errors::user::{
		AddUserError, DeleteUserError, GetUserByEmailError, GetUserError, UpdateUserError,
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
	// MARK: Dataset
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

	async fn get_user(&self, user: UserId) -> Result<UserInfo, GetUserError> {
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
			Err(sqlx::Error::RowNotFound) => Err(GetUserError::NotFound),
			Err(e) => Err(GetUserError::DbError(Box::new(e))),
			Ok(res) => Ok(UserInfo {
				id: res.get::<i64, _>("id").into(),
				name: res.get::<String, _>("user_name").into(),
				email: res.get::<String, _>("user_email").into(),
				password: serde_json::from_str(res.get::<&str, _>("user_pass")).unwrap(),
			}),
		};
	}

	async fn get_user_by_email(&self, email: &str) -> Result<UserInfo, GetUserByEmailError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| GetUserByEmailError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT * FROM users WHERE user_email=$1;")
			.bind(email)
			.fetch_one(&mut *conn)
			.await;

		return match res {
			Err(sqlx::Error::RowNotFound) => Err(GetUserByEmailError::NotFound),
			Err(e) => Err(GetUserByEmailError::DbError(Box::new(e))),
			Ok(res) => Ok(UserInfo {
				id: res.get::<i64, _>("id").into(),
				name: res.get::<String, _>("user_name").into(),
				email: res.get::<String, _>("user_email").into(),
				password: serde_json::from_str(res.get::<&str, _>("user_pass")).unwrap(),
			}),
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
}
