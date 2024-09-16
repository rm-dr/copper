use copper_util::logging::LoggingPreset;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use tracing::{debug, info};

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize)]
pub struct StoragedConfig {
	/// The logging level to run with
	#[serde(default)]
	pub storaged_loglevel: LoggingPreset,

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

	/// The secret used to authenticate callers.
	/// This should be a long sequence of random characters.
	///
	/// Anyone with this key can call all `pipelined` endpoints.
	pub storaged_secret: String,
}

impl StoragedConfig {
	pub fn default_request_body_limit() -> usize {
		2_000_000
	}
}

// Capture this in a module to modify log source
mod auth {
	use super::*;
	use axum::http::{HeaderMap, Uri};

	impl StoragedConfig {
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

			if token == self.storaged_secret {
				info!(message = "Authentication successful", ?uri,);
				return true;
			} else {
				info!(
					message = "Authentication failed",
					reason = "header mismatch",
					?uri,
					configured_secret = self.storaged_secret,
					received_secret = token
				);
				return false;
			}
		}
	}
}
