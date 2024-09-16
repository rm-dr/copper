#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

// We don't derive ToSchema here, since utoipa doesn't
// take serde(transparent) into account.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
