use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use super::GroupPermissions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum GroupId {
	RootGroup,
	Group { id: u32 },
}

impl GroupId {
	pub fn get_id(&self) -> Option<u32> {
		match self {
			Self::RootGroup => None,
			Self::Group { id } => Some(*id),
		}
	}
}

impl From<u32> for GroupId {
	fn from(value: u32) -> Self {
		Self::Group { id: value }
	}
}

#[derive(Debug, Clone)]
pub struct GroupInfo {
	pub id: GroupId,
	pub name: SmartString<LazyCompact>,
	pub permissions: GroupPermissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
