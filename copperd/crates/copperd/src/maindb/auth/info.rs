use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use smartstring::{LazyCompact, SmartString};
use std::{error::Error, fmt::Display, str::FromStr};
use time::OffsetDateTime;
use utoipa::ToSchema;

use super::GroupPermissions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GroupInfo {
	pub id: GroupId,
	pub parent: Option<GroupId>,

	#[schema(value_type =String)]
	pub name: SmartString<LazyCompact>,

	#[serde(skip)]
	pub permissions: GroupPermissions,
}

// TODO: We don't derive ToSchema here because it doesn't handle `transparent` correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug)]
pub enum UserColorParseError {
	BadStringLength,
	BadColorHex,
}

impl Display for UserColorParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BadStringLength => write!(f, "bad string length"),
			Self::BadColorHex => write!(f, "bad color hex"),
		}
	}
}

impl Error for UserColorParseError {}

#[derive(Debug, Clone, Copy, SerializeDisplay, DeserializeFromStr)]
pub struct UserColor {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl FromStr for UserColor {
	type Err = UserColorParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() != 7 {
			return Err(UserColorParseError::BadStringLength);
		}

		Ok(Self {
			r: u8::from_str_radix(&s[1..3], 16).map_err(|_| UserColorParseError::BadColorHex)?,
			g: u8::from_str_radix(&s[3..5], 16).map_err(|_| UserColorParseError::BadColorHex)?,
			b: u8::from_str_radix(&s[5..7], 16).map_err(|_| UserColorParseError::BadColorHex)?,
		})
	}
}

impl Display for UserColor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserInfo {
	#[schema(value_type = u32)]
	pub id: UserId,

	pub group: GroupInfo,

	pub name: String,
	pub email: Option<String>,

	#[schema(value_type = String)]
	pub color: UserColor,
}

#[derive(Debug, Clone)]
pub struct AuthToken {
	pub user: UserId,
	pub token: SmartString<LazyCompact>,

	/// When this token expires. If this is [`None`],
	/// this token does not expire.
	pub expires: Option<OffsetDateTime>,
}
