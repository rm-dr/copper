use copper_util::logging::LoggingPreset;
use serde::Deserialize;

/// `await` for this many ms between successive polls
/// of pipeline tasks. This constant is used in a few
/// different contexts, but it's purpose remains the same.
///
/// If this duration is too long, we'll waste time waiting
/// but if it is too short we'll cpu cycles checking
/// unfinished futures and switching tasks.
pub const ASYNC_POLL_AWAIT_MS: u64 = 1000;

/// Note that the field of this struct are not capitalized.
/// Envy is case-insensitive, and expects Rust fields to be snake_case.
#[derive(Debug, Deserialize, Clone)]
pub struct PiperConfig {
	/// The logging level to run with
	#[serde(default)]
	pub piper_loglevel: LoggingPreset,

	/// Object store key id
	pub piper_objectstore_key_id: String,
	/// Object store secret
	pub piper_objectstore_key_secret: String,
	/// Object store url
	pub piper_objectstore_url: String,
	/// Object store bucket
	pub piper_objectstore_bucket: String,

	/// The address of the item db this pipeline runner should use
	/// Looks like `postgres://user:pass@host/database`
	pub piper_itemdb_addr: String,

	/// The address of the jobqueue this pipeline runner should use
	/// Looks like `postgres://user:pass@host/database`
	pub piper_jobqueue_addr: String,

	/// The maximum size, in bytes, of a binary fragment in the pipeline.
	/// Smaller values slow down pipelines; larger values use more memory.
	#[serde(default = "PiperConfig::default_frag_size")]
	pub piper_blob_fragment_size: usize,

	/// The message capacity of binary stream channels.
	///
	/// Smaller values increase the probability of pipeline runs failing due to an
	/// overflowing channel, larger values use more memory.
	#[serde(default = "PiperConfig::default_channel_size")]
	pub piper_stream_channel_size: usize,

	/// How many pipeline jobs to run at once
	#[serde(default = "PiperConfig::default_max_running_jobs")]
	pub piper_max_running_jobs: usize,
}

impl PiperConfig {
	fn default_frag_size() -> usize {
		10_000_000
	}

	fn default_channel_size() -> usize {
		16
	}

	fn default_max_running_jobs() -> usize {
		4
	}
}
