use axum::http::HeaderMap;
use copper_util::logging::LoggingPreset;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use tracing::{debug, info};

/// `await` for this many ms between successive polls
/// of pipeline tasks. This constant is used in a few
/// different contexts, but it's purpose remains the same.
///
/// If this duration is too long, we'll waste time waiting
/// but if it is too short we'll cpu cycles checking
/// unfinished futures and switching tasks.
pub const ASYNC_POLL_AWAIT_MS: u64 = 10;

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize)]
pub struct PipelinedConfig {
	/// The logging level to run with
	#[serde(default)]
	pub pipelined_loglevel: LoggingPreset,

	/// The maximum size, in bytes, of a binary fragment in the pipeline.
	/// Smaller values slow down pipelines; larger values use more memory.
	#[serde(default = "PipelinedConfig::default_frag_size")]
	pub pipelined_blob_fragment_size: usize,

	/// The message capacity of binary stream channels.
	///
	/// Smaller values increase the probability of pipeline runs failing due to an
	/// overflowing channel, larger values use more memory.
	#[serde(default = "PipelinedConfig::default_channel_size")]
	pub pipelined_stream_channel_size: usize,

	/// How many pipeline jobs to run at once
	#[serde(default = "PipelinedConfig::default_max_running_jobs")]
	pub pipelined_max_running_jobs: usize,

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
	pub pipelined_storaged_addr: String,

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
	fn default_frag_size() -> usize {
		2_000_000
	}

	fn default_channel_size() -> usize {
		16
	}

	fn default_max_running_jobs() -> usize {
		4
	}

	fn default_request_body_limit() -> usize {
		2_000_000
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
