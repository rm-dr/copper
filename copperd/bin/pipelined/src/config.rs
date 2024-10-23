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

	pub pipelined_storage_db_addr: String,

	/// Object store key id
	pub pipelined_objectstore_key_id: String,
	/// Object store secret
	pub pipelined_objectstore_key_secret: String,
	/// Object store url
	pub pipelined_objectstore_url: String,
	/// Object store bucket
	pub pipelined_objectstore_bucket: String,

	pub pipelined_jobqueue_db: String,
}

impl PipelinedConfig {
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
