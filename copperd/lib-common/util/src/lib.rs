#![warn(missing_docs)]

//! Shared utilities used throughout the workspace

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod graph;
pub mod mime;

/// The types of hashes we support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[allow(missing_docs)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}
