use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, Row};

use super::{
	errors::{CreateGroupError, CreateUserError},
	MainDB,
};

const PW_TOKEN_LENGTH: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum SerializedGroupPermissionState {
	Transparent,
	Disallowed,
}

impl Default for SerializedGroupPermissionState {
	fn default() -> Self {
		Self::Transparent
	}
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
	t == &T::default()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SerializedGroupPermissions {
	#[serde(default, skip_serializing_if = "is_default")]
	edit_datasets: SerializedGroupPermissionState,

	#[serde(default, skip_serializing_if = "is_default")]
	edit_users: SerializedGroupPermissionState,

	#[serde(default, skip_serializing_if = "is_default")]
	edit_groups: SerializedGroupPermissionState,
}

#[derive(Debug, Clone)]
pub enum GroupPermissionState {
	Allowed,
	Disallowed { by: GroupId },
}

impl GroupPermissionState {
	fn overlay(&mut self, other: &SerializedGroupPermissionState, other_group: GroupId) {
		match (&self, other) {
			(Self::Allowed, SerializedGroupPermissionState::Transparent) => *self = Self::Allowed,
			(Self::Allowed, SerializedGroupPermissionState::Disallowed) => {
				*self = Self::Disallowed { by: other_group }
			}
			(Self::Disallowed { .. }, _) => return,
		}
	}

	pub fn allowed(&self) -> bool {
		matches!(self, Self::Allowed)
	}
}

#[derive(Debug, Clone)]
pub struct GroupPermissions {
	pub edit_datasets: GroupPermissionState,
	pub edit_users: GroupPermissionState,
	pub edit_groups: GroupPermissionState,
}

impl GroupPermissions {
	/// Make a new set of permissions for the root group. (i.e, where all actions are allowed)
	/// All permissions are created by overlaying parents on the root group.
	fn new_root() -> Self {
		Self {
			edit_datasets: GroupPermissionState::Allowed,
			edit_users: GroupPermissionState::Allowed,
			edit_groups: GroupPermissionState::Allowed,
		}
	}

	/// Modify this group by passing it through a filter.
	///
	/// Any permissions disallowed in `filter_group` will be disallowed in this group.
	/// Any permissions already disallowed in this group will not be changed.
	///
	/// `overlay` always produces a group that is "weaker-or-equal-to" `self`.
	fn overlay(mut self, filter: &SerializedGroupPermissions, filter_group: GroupId) -> Self {
		self.edit_datasets
			.overlay(&filter.edit_datasets, filter_group);
		self.edit_users.overlay(&filter.edit_datasets, filter_group);
		self.edit_groups
			.overlay(&filter.edit_datasets, filter_group);
		return self;
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId {
	id: u32,
}

impl From<GroupId> for u32 {
	fn from(value: GroupId) -> Self {
		value.id
	}
}

impl From<u32> for GroupId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone)]
pub struct GroupInfo {
	pub id: GroupId,
	pub parent: Option<GroupId>,
	pub name: SmartString<LazyCompact>,
	pub permissions: GroupPermissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId {
	id: u32,
}

impl From<UserId> for u32 {
	fn from(value: UserId) -> Self {
		value.id
	}
}

impl From<u32> for UserId {
	fn from(value: u32) -> Self {
		Self { id: value }
	}
}

#[derive(Debug, Clone)]
pub struct UserInfo {
	pub id: UserId,
	pub name: SmartString<LazyCompact>,
	pub group: GroupInfo,
}

#[derive(Debug, Clone)]
pub struct AuthToken {
	pub user: UserId,
	pub token: SmartString<LazyCompact>,
}

impl MainDB {
	#[inline(always)]
	pub fn generate_token(user: UserId) -> AuthToken {
		let rand: String = rand::thread_rng()
			.sample_iter(&Alphanumeric)
			.take(PW_TOKEN_LENGTH)
			.map(char::from)
			.collect();
		AuthToken {
			user,
			token: format!("copper_{}_{rand}", u32::from(user)).into(),
		}
	}

	pub async fn new_group(
		&self,
		name: &str,
		parent: Option<GroupId>,
	) -> Result<(), CreateGroupError> {
		// No empty names
		if name == "" {
			return Err(CreateGroupError::BadName(
				"Group name cannot be empty".into(),
			));
		} else if name.trim() == "" {
			return Err(CreateGroupError::BadName(
				"Group name cannot be whitespace".into(),
			));
		}

		let res = sqlx::query(
			"
			INSERT INTO groups (
				group_name, group_parent, group_permissions
			) VALUES (?, ?, ?);
			",
		)
		.bind(name)
		.bind(parent.map(u32::from))
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

	pub async fn del_group(&self, group: GroupId) -> Result<(), sqlx::Error> {
		// Start transaction
		let mut conn_lock = self.conn.lock().await;
		let mut t = conn_lock.begin().await?;

		sqlx::query("DELETE FROM groups WHERE group_parent=?;")
			.bind(u32::from(group))
			.execute(&mut *t)
			.await?;

		sqlx::query("DELETE FROM users WHERE user_group=?;")
			.bind(u32::from(group))
			.execute(&mut *t)
			.await?;

		sqlx::query("DELETE FROM groups WHERE id=?;")
			.bind(u32::from(group))
			.execute(&mut *t)
			.await?;

		t.commit().await?;

		Ok(())
	}

	pub async fn new_user(
		&self,
		user_name: &str,
		password: &str,
		group: GroupId,
	) -> Result<(), CreateUserError> {
		// No empty names
		if user_name == "" {
			return Err(CreateUserError::BadName("User name cannot be empty".into()));
		} else if user_name.trim() == "" {
			return Err(CreateUserError::BadName(
				"User name cannot be whitespace".into(),
			));
		}

		let pw_hash = {
			let salt = SaltString::generate(&mut OsRng);
			let argon2 = Argon2::default();
			argon2
				.hash_password(password.as_bytes(), &salt)
				.unwrap()
				.to_string()
		};

		let res = sqlx::query(
			"
			INSERT INTO users (
				user_name, user_group, pw_hash
			) VALUES (?, ?, ?);
			",
		)
		.bind(user_name)
		.bind(u32::from(group))
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
			let t = Self::generate_token(user_id);
			self.active_tokens.lock().await.push(t.clone());
			return Ok(Some(t));
		}

		return Ok(None);
	}

	pub async fn get_user(&self, user: UserId) -> Result<Option<UserInfo>, sqlx::Error> {
		let res = sqlx::query("SELECT id, user_name, user_group FROM users WHERE id=?;")
			.bind(u32::from(user))
			.fetch_one(&mut *self.conn.lock().await)
			.await;

		match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => return Err(e),
			Ok(res) => {
				// This should never fail, since we have a foreign key constraint
				let group = self
					.get_group(res.get::<u32, _>("user_group").into())
					.await?
					.unwrap();
				return Ok(Some(UserInfo {
					id: user,
					name: res.get::<&str, _>("user_name").into(),
					group,
				}));
			}
		}
	}

	pub async fn get_group(&self, group: GroupId) -> Result<Option<GroupInfo>, sqlx::Error> {
		let res = sqlx::query(
			"SELECT id, group_name, group_parent, group_permissions FROM groups WHERE id=?;",
		)
		.bind(u32::from(group))
		.fetch_one(&mut *self.conn.lock().await)
		.await;

		match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
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
					.bind(u32::from(p))
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

				return Ok(Some(GroupInfo {
					id: group,
					name: res.get::<&str, _>("group_name").into(),
					parent: res.get::<Option<u32>, _>("group_parent").map(|x| x.into()),
					permissions,
				}));
			}
		}
	}
}
