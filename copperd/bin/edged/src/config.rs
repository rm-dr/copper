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

	/// The address of the database this storage server uses.
	/// Must be postgres.
	pub edged_db_addr: SmartString<LazyCompact>,

	/// IP and port of the `storaged` daemon we'll use
	/// Should look like `http://127.0.0.1:3030`
	pub edged_storaged_addr: String,

	/// The secret used to authenticate calls to storaged.
	pub edged_storaged_secret: String,

	/// IP and port of the `pipelined` daemon we'll use
	/// Should look like `http://127.0.0.1:3030`
	pub edged_pipelined_addr: String,

	/// The secret used to authenticate calls to pipelined.
	pub edged_pipelined_secret: String,

	/// Object store key id
	pub edged_objectstore_key_id: String,
	/// Object store secret
	pub edged_objectstore_key_secret: String,
	/// Object store url
	pub edged_objectstore_url: String,
	/// Object store bucket
	pub edged_objectstore_bucket: String,

	/// How long an upload job may idle before being deleted, in seconds
	pub edged_upload_job_timeout: u64,
}

impl EdgedConfig {
	pub fn default_request_body_limit() -> usize {
		2_000_000
	}
}
