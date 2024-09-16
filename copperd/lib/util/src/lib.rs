//! Shared utilities used throughout the workspace

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod env;
pub use env::*;

mod mime;
pub use mime::*;

pub mod graph;
pub mod logging;
pub mod names;

/// The types of hashes we support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[allow(missing_docs)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}
