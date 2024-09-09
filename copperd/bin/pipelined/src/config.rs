use copper_util::LogLevel;
use reqwest::Url;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};

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
			format!("pipelined={}", LogLevel::Info),
			LogLevel::Warn.to_string(),
		]
		.join(",")
	}
}
