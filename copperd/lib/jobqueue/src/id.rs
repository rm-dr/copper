#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

// We don't derive ToSchema here, since utoipa doesn't
// take serde(transparent) into account.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueuedJobId {
	id: SmartString<LazyCompact>,
}

impl QueuedJobId {
	pub fn as_str(&self) -> &str {
		&self.id
	}
}

impl From<&str> for QueuedJobId {
	fn from(value: &str) -> Self {
		Self { id: value.into() }
	}
}
