use std::fmt::Display;

use smartstring::{LazyCompact, SmartString};

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

#[derive(Debug, Clone)]
pub struct StoragedConfig {
	/// IP and port to bind to
	/// Should look like `127.0.0.1:3030`
	pub server_addr: SmartString<LazyCompact>,

	/// The address of the database this storage server uses
	pub db_addr: SmartString<LazyCompact>,

	/// Maximum request body size, in bytes
	/// If you're using a reverse proxy, make sure it
	/// also accepts requests of this size!
	pub request_body_limit: usize,
}

impl Default for StoragedConfig {
	fn default() -> Self {
		Self {
			request_body_limit: Self::default_request_body_limit(),
			server_addr: "127.0.0.1:5000".into(),
			db_addr: "sqlite://./data/test.sqlite?mode=rwc".into(),
		}
	}
}

impl StoragedConfig {
	fn default_request_body_limit() -> usize {
		2_000_000
	}

	/// Convert this logging config to a tracing env filter
	pub fn to_env_filter(&self) -> String {
		[
			format!("sqlx={}", LogLevel::Warn),
			format!("tower_http={}", LogLevel::Warn),
			format!("storaged={}", LogLevel::Info),
			LogLevel::Warn.to_string(),
		]
		.join(",")
	}
}
