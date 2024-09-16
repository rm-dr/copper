#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// We don't derive ToSchema here, since utoipa doesn't
// take serde(transparent) into account.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId {
	id: i64,
}

impl From<UserId> for i64 {
	fn from(value: UserId) -> Self {
		value.id
	}
}

impl From<i64> for UserId {
	fn from(value: i64) -> Self {
		Self { id: value }
	}
}
