use copper_util::LogLevel;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize)]
pub struct StoragedConfig {
	/// Maximum request body size, in bytes
	/// If you're using a reverse proxy, make sure it
	/// also accepts requests of this size!
	#[serde(default = "StoragedConfig::default_request_body_limit")]
	pub storaged_request_body_limit: usize,

	/// IP and port to bind to
	/// Should look like `127.0.0.1:3030`
	pub storaged_server_addr: SmartString<LazyCompact>,

	/// The address of the database this storage server uses
	pub storaged_db_addr: SmartString<LazyCompact>,
}

impl StoragedConfig {
	pub fn default_request_body_limit() -> usize {
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
