//! Shared utilities used throughout the workspace

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod env;
pub use env::*;

mod mime;
pub use mime::*;

pub mod graph;

/// The types of hashes we support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[allow(missing_docs)]
pub enum HashType {
	MD5,
	SHA256,
	SHA512,
}

#[derive(Debug)]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl Default for LogLevel {
	fn default() -> Self {
		Self::Info
	}
}

impl Display for LogLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Trace => write!(f, "trace"),
			Self::Debug => write!(f, "debug"),
			Self::Info => write!(f, "info"),
			Self::Warn => write!(f, "warn"),
			Self::Error => write!(f, "error"),
		}
	}
}
