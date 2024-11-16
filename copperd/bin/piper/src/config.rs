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
	pub piper_objectstore_storage_bucket: String,

	/// The address of the item db this pipeline runner should use
	/// Looks like `postgres://user:pass@host/database`
	pub piper_itemdb_addr: String,

	/// The address of the jobqueue this pipeline runner should use
	/// Looks like `postgres://user:pass@host/database`
	pub piper_jobqueue_addr: String,

	/// The maximum size, in bytes, of a stream fragment in the pipeline.
	/// Smaller values slow down pipelines; larger values use more memory.
	#[serde(default = "PiperConfig::default_frag_size")]
	pub piper_stream_fragment_size: usize,

	/// The maximum size, in bytes, of a stream processing channel.
	/// Stream channels hold stream fragments, which contain at most `stream_fragment_size` bytes.
	#[serde(default = "PiperConfig::default_chan_size")]
	pub piper_stream_channel_size: usize,

	/// The number of pipeline jobs to run in parallel
	#[serde(default = "PiperConfig::default_parallel_jobs")]
	pub piper_parallel_jobs: usize,
}

impl PiperConfig {
	fn default_frag_size() -> usize {
		10_000_000
	}

	fn default_chan_size() -> usize {
		10
	}

	fn default_parallel_jobs() -> usize {
		std::thread::available_parallelism()
			.map(|x| x.into())
			.unwrap_or(4)
	}
}
