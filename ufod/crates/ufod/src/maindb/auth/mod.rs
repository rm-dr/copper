use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
	http::{header::SET_COOKIE, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::{
	cookie::{Cookie, Expiration, SameSite},
	CookieJar,
};
use errors::{CreateGroupError, CreateUserError, DeleteGroupError};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use sqlx::{Row, SqlitePool};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

pub mod errors;
mod info;
mod permissions;
pub use info::*;
pub use permissions::*;

const AUTH_TOKEN_LENGTH: usize = 32;
pub const AUTH_COOKIE_NAME: &str = "authtoken";

pub struct AuthProvider {
	pool: SqlitePool,
	active_tokens: Mutex<Vec<AuthToken>>,
}

impl AuthProvider {
	#[inline(always)]
	async fn generate_token(&self, user: UserId) -> AuthToken {
		let token = 'outer: loop {
			let rand: String = rand::thread_rng()
				.sample_iter(&Alphanumeric)
				.take(AUTH_TOKEN_LENGTH)
				.map(char::from)
				.collect();
			let token = format!("copper_{}_{rand}", u32::from(user));

			// Make sure token isn't already used
			for t in self.active_tokens.lock().await.iter() {
				if t.token == token {
					continue 'outer;
				}
			}
			break token;
		};

		AuthToken {
			user,
			token: token.into(),
			// TODO: config
			expires: OffsetDateTime::now_utc()
				.checked_add(Duration::days(7))
				.unwrap(),
		}
	}

	pub(super) fn new(pool: SqlitePool) -> Self {
		Self {
			pool,
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
	/// Match a user to an authentication token or log out.
	/// This simplifies api code, and automatically logs users out if their token is invalid.
	pub async fn auth_or_logout(&self, jar: &CookieJar) -> Result<UserInfo, Response> {
		match self.check_cookies(jar).await {
			Ok(None) => {}
			Ok(Some(u)) => return Ok(u),
			Err(e) => {
				error!(
					message = "Could not check auth cookies",
					cookies = ?jar,
					error = ?e
				);
				return Err((
					StatusCode::INTERNAL_SERVER_ERROR,
					"Could not check auth cookies",
				)
					.into_response());
			}
		}

		// If cookie is invalid, clean up and delete client cookies
		self.terminate_session(jar).await;
		let cookie = Cookie::build((AUTH_COOKIE_NAME, ""))
			.path("/")
			.secure(true)
			.http_only(true)
			.same_site(SameSite::None)
			.expires(Expiration::from(OffsetDateTime::UNIX_EPOCH));

		return Err((
			StatusCode::UNAUTHORIZED,
			AppendHeaders([(SET_COOKIE, cookie.to_string())]),
			"Invalid auth cookie, logging out",
		)
			.into_response());
	}

	pub async fn check_cookies(&self, jar: &CookieJar) -> Result<Option<UserInfo>, sqlx::Error> {
		let token = if let Some(h) = jar.get(AUTH_COOKIE_NAME) {
			h.value()
		} else {
			return Ok(None);
		};

		for t in self.active_tokens.lock().await.iter() {
			if t.token == token {
				// Expired logins are invalid
				// These will be cleaned up by `auth_or_logout`
				// (if the browser doesn't do so automatically)
				if t.expires < OffsetDateTime::now_utc() {
					return Ok(None);
				}

				return match self.get_user(t.user).await {
					Ok(user) => Ok(Some(user)),
					Err(sqlx::Error::RowNotFound) => {
						// Tried to authenticate with a user that doesn't exist.
						// This probably happened because our user was deleted.
						// Invalidate this session and return None.
						self.terminate_session(jar).await;

						debug!(
							message = "Tried to authenticate as a user that doesn't exist",
							cookies = ?jar
						);

						Ok(None)
					}
					Err(e) => Err(e),
				};
			}
		}

		return Ok(None);
	}

	pub async fn terminate_session(&self, jar: &CookieJar) -> Option<AuthToken> {
		let token = if let Some(h) = jar.get(AUTH_COOKIE_NAME) {
			h.value()
		} else {
			return None;
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

		trace!(
			message = "Deleted session",
			token = ?x
		);

		return x;
	}

	pub async fn try_auth_user(
		&self,
		user_name: &str,
		password: &str,
	) -> Result<Option<AuthToken>, sqlx::Error> {
		info!(
			message = "Received login request",
			user = user_name,
			password = password,
		);

		let mut conn = self.pool.acquire().await?;
		let (user_id, pw_hash): (UserId, String) = {
			let res = sqlx::query("SELECT id, pw_hash FROM users WHERE user_name=?;")
				.bind(user_name)
				.fetch_one(&mut *conn)
				.await;

			match res {
				Err(sqlx::Error::RowNotFound) => {
					info!(
						message = "Login failed: user not found",
						user = user_name,
						password = password,
					);

					return Ok(None);
				}
				Err(e) => {
					error!(
						message = "Login failed: error",
						user = user_name,
						password = password,
						error = ?e,
					);

					return Err(e);
				}
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

			info!(
				message = "Login success",
				user = user_name,
				password = password,
				new_token = ?t.token
			);

			return Ok(Some(t));
		}

		info!(
			message = "Login failed: did not pass verification",
			user = user_name,
			password = password,
		);
		return Ok(None);
	}

	pub async fn new_group(&self, name: &str, parent: GroupId) -> Result<(), CreateGroupError> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| CreateGroupError::DbError(Box::new(e)))?;

		// No empty names
		let name = name.trim();
		if name.is_empty() {
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
		.execute(&mut *conn)
		.await;

		info!(message = "Created new group", name, ?parent);

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

	pub async fn del_group(&self, del_group: GroupId) -> Result<(), DeleteGroupError> {
		if del_group == GroupId::RootGroup {
			return Err(DeleteGroupError::CantDeleteRootGroup);
		}

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;
		let res = sqlx::query("SELECT id FROM groups ORDER BY id;")
			.bind(del_group.get_id())
			.fetch_all(&mut *conn)
			.await
			.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

		// Reverse order so we delete child groups first
		for row in res.into_iter().rev() {
			let group: GroupId = row.get::<u32, _>("id").into();
			if del_group == group
				|| self
					.is_group_parent(del_group, group)
					.await
					.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?
			{
				sqlx::query("DELETE FROM users WHERE user_group=?;")
					.bind(group.get_id())
					.execute(&mut *conn)
					.await
					.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

				sqlx::query("DELETE FROM groups WHERE id=?;")
					.bind(group.get_id())
					.execute(&mut *conn)
					.await
					.map_err(|e| DeleteGroupError::DbError(Box::new(e)))?;

				info!(
					message = "Deleted group",
					group_id = ?group,
				)
			}
		}

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
		if user_name.is_empty() {
			return Err(CreateUserError::BadName("User name cannot be empty".into()));
		}

		let pw_hash = Self::hash_password(password);

		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| CreateUserError::DbError(Box::new(e)))?;
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
		.execute(&mut *conn)
		.await;

		info!(
			message = "Created a user",
			user = user_name,
			password,
			?group,
		);

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
		let mut conn = self.pool.acquire().await?;
		sqlx::query("DELETE FROM users WHERE id=?;")
			.bind(u32::from(user))
			.execute(&mut *conn)
			.await?;

		// Invalidate all sessions of the user we just deleted
		let mut active_tokens = self.active_tokens.lock().await;
		let mut i = 0;
		while i < active_tokens.len() {
			if active_tokens[i].user == user {
				active_tokens.swap_remove(i);
			} else {
				i += 1;
			}
		}

		info!(
			message = "Deleted a user",
			user_id = ?user,
		);

		Ok(())
	}

	pub async fn get_user(&self, user: UserId) -> Result<UserInfo, sqlx::Error> {
		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query("SELECT id, user_name, user_group FROM users WHERE id=?;")
			.bind(u32::from(user))
			.fetch_one(&mut *conn)
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
				parent: None,
				permissions: GroupPermissions::new_root(),
			});
		}

		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query(
			"SELECT id, group_name, group_parent, group_permissions FROM groups WHERE id=?;",
		)
		.bind(group.get_id())
		.fetch_one(&mut *conn)
		.await?;
		let first_parent: Option<GroupId> =
			res.get::<Option<u32>, _>("group_parent").map(|x| x.into());

		// Collect permissions from all parents
		let mut permissions: Vec<(GroupId, SerializedGroupPermissions)> = Vec::new();
		let mut parent = first_parent;
		while let Some(p) = parent {
			let r = sqlx::query("SELECT group_parent, group_permissions FROM groups WHERE id=?;")
				.bind(p.get_id())
				.fetch_one(&mut *conn)
				.await?;

			permissions.push((
				p,
				serde_json::from_str(r.get::<&str, _>("group_permissions")).unwrap(),
			));
			parent = r.get::<Option<u32>, _>("group_parent").map(|x| x.into());
		}

		// Resolve permissions from the bottom up
		let permissions = permissions
			.into_iter()
			.fold(GroupPermissions::new_root(), |a, (group, perms)| {
				a.overlay(&perms, group)
			});

		return Ok(GroupInfo {
			id: group,
			name: res.get::<&str, _>("group_name").into(),
			permissions,
			parent: first_parent,
		});
	}

	pub async fn list_groups(&self, starting_from: GroupId) -> Result<Vec<GroupInfo>, sqlx::Error> {
		// A child group cannot be created after its parent,
		// so this method will always list parent groups before child groups.
		// UI depends on this, as do some other `AuthProvider` methods.
		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query("SELECT id, group_parent FROM groups ORDER BY id;")
			.fetch_all(&mut *conn)
			.await?;

		let mut out = Vec::new();
		for group in [GroupId::RootGroup]
			.into_iter()
			.chain(res.into_iter().map(|row| row.get::<u32, _>("id").into()))
		{
			if group == starting_from || self.is_group_parent(starting_from, group).await? {
				out.push(self.get_group(group).await?)
			}
		}

		return Ok(out);
	}

	pub async fn list_users(&self, in_group: GroupId) -> Result<Vec<UserInfo>, sqlx::Error> {
		let mut conn = self.pool.acquire().await?;
		let res = if in_group == GroupId::RootGroup {
			sqlx::query("SELECT id FROM users WHERE user_group IS NULL ORDER BY id;")
				.fetch_all(&mut *conn)
				.await?
		} else {
			sqlx::query("SELECT id FROM users WHERE user_group=? ORDER BY id;")
				.bind(in_group.get_id())
				.fetch_all(&mut *conn)
				.await?
		};

		let mut out = Vec::new();
		for row in res {
			out.push(self.get_user(row.get::<u32, _>("id").into()).await?);
		}

		return Ok(out);
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

		let mut conn = self.pool.acquire().await?;
		let res = sqlx::query("SELECT group_parent FROM groups WHERE id=?;")
			.bind(child.get_id())
			.fetch_one(&mut *conn)
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
						.fetch_one(&mut *conn)
						.await?;

					last_parent = r.get::<Option<u32>, _>("group_parent").map(|x| x.into());
				}

				return Ok(false);
			}
		}
	}
}
