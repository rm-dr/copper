use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum_extra::extract::CookieJar;
use errors::{CreateGroupError, CreateUserError, DeleteGroupError};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use sqlx::{Connection, Row, SqliteConnection};
use tokio::sync::Mutex;

pub mod errors;
mod info;
mod permissions;
pub use info::*;
pub use permissions::*;

const AUTH_TOKEN_LENGTH: usize = 32;
pub const AUTH_COOKIE_NAME: &'static str = "authtoken";

pub struct AuthProvider {
	conn: Arc<Mutex<SqliteConnection>>,
	active_tokens: Mutex<Vec<AuthToken>>,
}

impl AuthProvider {
	#[inline(always)]
	async fn generate_token(&self, user: UserId) -> AuthToken {
		let token = loop {
			let rand: String = rand::thread_rng()
				.sample_iter(&Alphanumeric)
				.take(AUTH_TOKEN_LENGTH)
				.map(char::from)
				.collect();
			let token = format!("copper_{}_{rand}", u32::from(user));

			// Make sure token isn't already used
			for t in self.active_tokens.lock().await.iter() {
				if t.token == token {
					break;
				}
			}
			break token;
		};

		AuthToken {
			user,
			token: token.into(),
		}
	}

	pub(super) fn new(conn: Arc<Mutex<SqliteConnection>>) -> Self {
		Self {
			conn,
			active_tokens: Mutex::new(Vec::new()),
		}
	}

	pub fn hash_password(password: &str) -> String {
		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::default();
		argon2
			.hash_password(password.as_bytes(), &salt)
			.unwrap()
			.to_string()
	}
}

impl AuthProvider {
	pub async fn check_cookies(&self, jar: &CookieJar) -> Result<Option<UserInfo>, sqlx::Error> {
		let token = if let Some(h) = jar.get(AUTH_COOKIE_NAME) {
			h.value()
		} else {
			return Ok(None);
		};

		for t in self.active_tokens.lock().await.iter() {
			if t.token == token {
				return Ok(Some(self.get_user(t.user).await?));
			}
		}

		return Ok(None);
	}

	pub async fn terminate_session(&self, jar: &CookieJar) -> Result<Option<AuthToken>, ()> {
		let token = if let Some(h) = jar.get(AUTH_COOKIE_NAME) {
			h.value()
		} else {
			return Ok(None);
		};

		let mut active_tokens = self.active_tokens.lock().await;
		let mut i = 0;
		let mut x = None;
		while i < active_tokens.len() {
			if active_tokens[i].token == token {
				x = Some(active_tokens.swap_remove(i));
			} else {
				i += 1;
			}
		}

		return Ok(x);
	}

	pub async fn try_auth_user(
		&self,
		user_name: &str,
		password: &str,
	) -> Result<Option<AuthToken>, sqlx::Error> {
		let (user_id, pw_hash): (UserId, String) = {
			let res = sqlx::query("SELECT id, pw_hash FROM users WHERE user_name=?;")
				.bind(user_name)
				.fetch_one(&mut *self.conn.lock().await)
				.await;

			match res {
				Err(sqlx::Error::RowNotFound) => return Ok(None),
				Err(e) => return Err(e),
				Ok(res) => (
					res.get::<u32, _>("id").into(),
					res.get::<String, _>("pw_hash"),
				),
			}
		};

		let parsed_hash = PasswordHash::new(&pw_hash).unwrap();
		let verify = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);

		if verify.is_ok() {
			let t = self.generate_token(user_id).await;
			self.active_tokens.lock().await.push(t.clone());
			return Ok(Some(t));
		}

		return Ok(None);
	}

	pub async fn new_group(&self, name: &str, parent: GroupId) -> Result<(), CreateGroupError> {
		// No empty names
		let name = name.trim();
		if name == "" {
			return Err(CreateGroupError::BadName(
				"Group name cannot be empty".into(),
			));
		} else if name == "Root Group" {
			return Err(CreateGroupError::AlreadyExists);
		}

		let res = sqlx::query(
			"
			INSERT INTO groups (
				group_name, group_parent, group_permissions
			) VALUES (?, ?, ?);
			",
		)
		.bind(name)
		.bind(parent.get_id())
		.bind(serde_json::to_string(&SerializedGroupPermissions::default()).unwrap())
		.execute(&mut *self.conn.lock().await)
		.await;

		match res {
			Ok(_) => return Ok(()),
			Err(e) => {
				if let Some(e) = e.as_database_error() {
					if e.is_unique_violation() {
						return Err(CreateGroupError::AlreadyExists);
					} else if e.is_foreign_key_violation() {
						return Err(CreateGroupError::BadParent);
					}
				}
				return Err(CreateGroupError::DbError(Box::new(e)));
			}
		}
	}

	pub async fn del_group(&self, group: GroupId) -> Result<(), DeleteGroupError> {
		if group == GroupId::RootGroup {
			return Err(DeleteGroupError::CantDeleteRootGroup);
		}

		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock
			.begin()
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

		sqlx::query("DELETE FROM groups WHERE group_parent=?;")
			.bind(group.get_id())
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

		sqlx::query("DELETE FROM users WHERE user_group=?;")
			.bind(group.get_id())
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

		sqlx::query("DELETE FROM groups WHERE id=?;")
			.bind(group.get_id())
			.execute(&mut *t)
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

		t.commit()
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

		Ok(())
	}

	pub async fn new_user(
		&self,
		user_name: &str,
		password: &str,
		group: GroupId,
	) -> Result<(), CreateUserError> {
		// No empty names
		let user_name = user_name.trim();
		if user_name == "" {
			return Err(CreateUserError::BadName("User name cannot be empty".into()));
		}

		let pw_hash = Self::hash_password(password);
		let res = sqlx::query(
			"
			INSERT INTO users (
				user_name, user_group, pw_hash
			) VALUES (?, ?, ?);
			",
		)
		.bind(user_name)
		.bind(group.get_id())
		.bind(pw_hash)
		.execute(&mut *self.conn.lock().await)
		.await;

		match res {
			Ok(_) => return Ok(()),
			Err(e) => {
				if let Some(e) = e.as_database_error() {
					if e.is_foreign_key_violation() {
						return Err(CreateUserError::BadGroup);
					} else if e.is_unique_violation() {
						return Err(CreateUserError::AlreadyExists);
					}
				}
				return Err(CreateUserError::DbError(Box::new(e)));
			}
		}
	}

	pub async fn del_user(&self, user: UserId) -> Result<(), sqlx::Error> {
		sqlx::query("DELETE FROM users WHERE id=?;")
			.bind(u32::from(user))
			.execute(&mut *self.conn.lock().await)
			.await?;
		Ok(())
	}

	pub async fn get_user(&self, user: UserId) -> Result<UserInfo, sqlx::Error> {
		let res = sqlx::query("SELECT id, user_name, user_group FROM users WHERE id=?;")
			.bind(u32::from(user))
			.fetch_one(&mut *self.conn.lock().await)
			.await;

		match res {
			Err(e) => return Err(e),
			Ok(res) => {
				// This should never fail, since we have a foreign key constraint
				let group = self
					.get_group(
						res.get::<Option<u32>, _>("user_group")
							.map(|x| x.into())
							.unwrap_or(GroupId::RootGroup),
					)
					.await?;
				return Ok(UserInfo {
					id: user,
					name: res.get::<&str, _>("user_name").into(),
					group,
				});
			}
		}
	}

	pub async fn get_group(&self, group: GroupId) -> Result<GroupInfo, sqlx::Error> {
		if group == GroupId::RootGroup {
			return Ok(GroupInfo {
				id: group,
				name: "Root Group".into(),
				permissions: GroupPermissions::new_root(),
			});
		}

		let res = sqlx::query(
			"SELECT id, group_name, group_parent, group_permissions FROM groups WHERE id=?;",
		)
		.bind(group.get_id())
		.fetch_one(&mut *self.conn.lock().await)
		.await;

		match res {
			Err(e) => return Err(e),
			Ok(res) => {
				// Collect permissions from all parents
				let mut permissions: Vec<(GroupId, SerializedGroupPermissions)> = Vec::new();
				let mut parent: Option<GroupId> =
					res.get::<Option<u32>, _>("group_parent").map(|x| x.into());
				while let Some(p) = parent {
					let r = sqlx::query(
						"SELECT group_parent, group_permissions FROM groups WHERE id=?;",
					)
					.bind(p.get_id())
					.fetch_one(&mut *self.conn.lock().await)
					.await?;

					permissions.push((
						p,
						serde_json::from_str(r.get::<&str, _>("group_permissions")).unwrap(),
					));
					parent = r.get::<Option<u32>, _>("group_parent").map(|x| x.into());
				}

				// Resolve permissions from the bottom-up
				let permissions = permissions
					.into_iter()
					.fold(GroupPermissions::new_root(), |a, (group, perms)| {
						a.overlay(&perms, group)
					});

				return Ok(GroupInfo {
					id: group,
					name: res.get::<&str, _>("group_name").into(),
					permissions,
				});
			}
		}
	}

	pub async fn is_group_parent(
		&self,
		parent: GroupId,
		child: GroupId,
	) -> Result<bool, sqlx::Error> {
		if child == GroupId::RootGroup {
			return Ok(false);
		}
		if parent == GroupId::RootGroup {
			return Ok(true);
		}

		let res = sqlx::query("SELECT group_parent FROM groups WHERE id=?;")
			.bind(child.get_id())
			.fetch_one(&mut *self.conn.lock().await)
			.await;

		match res {
			Err(e) => return Err(e),
			Ok(res) => {
				let mut last_parent: Option<GroupId> =
					res.get::<Option<u32>, _>("group_parent").map(|x| x.into());
				while let Some(p) = last_parent {
					if p == parent {
						return Ok(true);
					}

					let r = sqlx::query("SELECT group_parent FROM groups WHERE id=?;")
						.bind(p.get_id())
						.fetch_one(&mut *self.conn.lock().await)
						.await?;

					last_parent = r.get::<Option<u32>, _>("group_parent").map(|x| x.into());
				}

				return Ok(false);
			}
		}
	}
}
