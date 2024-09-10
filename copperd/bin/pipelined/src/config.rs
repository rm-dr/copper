use axum::http::HeaderMap;
use copper_util::LogLevel;
use reqwest::Url;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use tracing::{debug, info};

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize)]
pub struct PipelinedConfig {
	/// The maximum size, in bytes, of a binary fragment in the pipeline.
	/// Smaller values slow down pipelines; larger values use more memory.
	#[serde(default = "PipelinedConfig::default_frag_size")]
	pub pipelined_blob_fragment_size: u64,

	/// How many pipeline jobs to run at once
	#[serde(default = "PipelinedConfig::default_parallel_jobs")]
	pub pipelined_parallel_jobs: usize,

	/// How many threads each job may use
	#[serde(default = "PipelinedConfig::default_job_threads")]
	pub pipelined_threads_per_job: usize,

	/// Maximum request body size, in bytes
	/// If you're using a reverse proxy, make sure it
	/// also accepts requests of this size!
	#[serde(default = "PipelinedConfig::default_request_body_limit")]
	pub pipelined_request_body_limit: usize,

	/// IP and port to bind to
	/// Should look like `127.0.0.1:3030`
	pub pipelined_server_addr: SmartString<LazyCompact>,

	/// IP and port of the `storaged` daemon we'll use
	/// Should look like `http://127.0.0.1:3030`
	pub pipelined_storaged_addr: Url,

	/// The secret used to authenticate calls to storaged.
	pub pipelined_storaged_secret: String,

	/// The secret used to authenticate callers.
	/// This should be a long sequence of random characters.
	///
	/// Anyone with this key can call all `pipelined` endpoints.
	pub pipelined_secret: String,

	/// Object store key id
	pub pipelined_objectstore_key_id: String,
	/// Object store secret
	pub pipelined_objectstore_key_secret: String,
	/// Object store url
	pub pipelined_objectstore_url: String,
	/// Object store bucket
	pub pipelined_objectstore_bucket: String,
}

impl PipelinedConfig {
	fn default_frag_size() -> u64 {
		2_000_000
	}

	fn default_parallel_jobs() -> usize {
		// TODO: detect using threads
		4
	}

	fn default_job_threads() -> usize {
		// TODO: detect using threads
		4
	}

	fn default_request_body_limit() -> usize {
		2_000_000
	}

	/// Convert this logging config to a tracing env filter
	pub fn to_env_filter(&self) -> String {
		[
			format!("tower_http={}", LogLevel::Warn),
			format!("s3={}", LogLevel::Warn),
			format!("aws_sdk_s3={}", LogLevel::Warn),
			format!("aws_smithy_runtime={}", LogLevel::Warn),
			format!("aws_smithy_runtime_api={}", LogLevel::Warn),
			format!("aws_sigv4={}", LogLevel::Warn),
			format!("hyper={}", LogLevel::Warn),
			format!("pipelined={}", LogLevel::Trace),
			LogLevel::Trace.to_string(),
		]
		.join(",")
	}
}

// Capture this in a module to modify log source
mod auth {
	use super::*;
	use axum::http::Uri;

	impl PipelinedConfig {
		/// Check the given header map for `self.pipelined_secret`.
		///
		/// Returns `true` if authentication is successful and `false` otherwise.
		pub fn header_has_valid_auth(&self, uri: &Uri, headers: &HeaderMap) -> bool {
			let token = if let Some(header) = headers.get("authorization") {
				match header.to_str().map(|x| x.strip_prefix("Bearer ")) {
					Ok(Some(secret)) => secret,
					Ok(None) => {
						debug!(
							message = "Authentication failed",
							reason = "invalid header value",
							?uri,
							?header,
						);
						return false;
					}
					Err(error) => {
						debug!(
							message = "Authentication failed",
							reason = "could not stringify auth header",
							?uri,
							?header,
							?error,
						);
						return false;
					}
				}
			} else {
				info!(
					message = "Authentication failed",
					reason = "header missing",
					?uri
				);
				return false;
			};

			if token == self.pipelined_secret {
				info!(message = "Authentication successful", ?uri,);
				return true;
			} else {
				info!(
					message = "Authentication failed",
					reason = "header mismatch",
					?uri,
					configured_secret = self.pipelined_secret,
					received_secret = token
				);
				return false;
			}
		}
	}
}
