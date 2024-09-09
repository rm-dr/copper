use std::fmt::Display;

use smartstring::{LazyCompact, SmartString};

#[derive(Debug)]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

impl Default for LogLevel {
	fn default() -> Self {
		Self::Info
	}
}

impl Display for LogLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Trace => write!(f, "trace"),
			Self::Debug => write!(f, "debug"),
			Self::Info => write!(f, "info"),
			Self::Warn => write!(f, "warn"),
			Self::Error => write!(f, "error"),
		}
	}
}

pub struct PipelinedConfig {
	/// The maximum size, in bytes, of a binary fragment in the pipeline.
	/// Smaller values slow down pipelines; larger values use more memory.
	pub blob_fragment_size: u64,

	/// How many pipeline jobs to run at once
	pub parallel_jobs: usize,

	/// How many threads each job may use
	pub threads_per_job: usize,

	/// IP and port to bind to
	/// Should look like `127.0.0.1:3030`
	pub server_addr: SmartString<LazyCompact>,

	/// Maximum request body size, in bytes
	/// If you're using a reverse proxy, make sure it
	/// also accepts requests of this size!
	pub request_body_limit: usize,
}

impl Default for PipelinedConfig {
	fn default() -> Self {
		Self {
			blob_fragment_size: Self::default_frag_size(),
			parallel_jobs: Self::default_parallel_jobs(),
			threads_per_job: Self::default_job_threads(),
			request_body_limit: Self::default_request_body_limit(),
			server_addr: "127.0.0.1:4000".into(),
		}
	}
}

impl PipelinedConfig {
	fn default_frag_size() -> u64 {
		2_000_000
	}

	fn default_parallel_jobs() -> usize {
		4
	}

	fn default_job_threads() -> usize {
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
