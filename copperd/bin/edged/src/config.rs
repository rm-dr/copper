use copper_util::logging::LoggingPreset;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize)]
pub struct EdgedConfig {
	/// The logging level to run with
	#[serde(default)]
	pub edged_loglevel: LoggingPreset,

	/// Maximum request body size, in bytes
	/// If you're using a reverse proxy, make sure it
	/// also accepts requests of this size!
	#[serde(default = "EdgedConfig::default_request_body_limit")]
	pub edged_request_body_limit: usize,

	/// IP and port to bind to
	/// Should look like `127.0.0.1:3030`
	pub edged_server_addr: SmartString<LazyCompact>,

	/// The address of the database this storage server uses
	pub edged_db_addr: SmartString<LazyCompact>,
}

impl EdgedConfig {
	pub fn default_request_body_limit() -> usize {
		2_000_000
	}
}
