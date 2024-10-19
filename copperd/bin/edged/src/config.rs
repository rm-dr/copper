use copper_util::logging::LoggingPreset;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use tracing::error;

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize, Clone)]
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
	/// The bucket to store user uploads in
	pub edged_objectstore_upload_bucket: String,

	/// How long an upload job may idle before being deleted, in seconds
	/// - if a pending upload job does not receive a part for this many seconds, it is deleted
	/// - if a finished upload job is not passed to a `run()` call within this many seconds, it is deleted
	pub edged_upload_job_timeout: u64,
}

impl EdgedConfig {
	fn default_request_body_limit() -> usize {
		10_000_000
	}

	/// Validate this config, logging and fixing errors.
	pub fn validate(mut self) -> Self {
		// Enforce minimum request body limit
		// (S3 multipart uploads have a 5MiB min part size)
		if self.edged_request_body_limit < 6_000_000 {
			error!(
				message = "EDGED_REQUEST_BODY_LIMIT is too small, setting minimum",
				value = self.edged_request_body_limit,
				minimum = 6_000_000
			);

			self.edged_request_body_limit = 6_000_000;
		}

		return self;
	}
}
