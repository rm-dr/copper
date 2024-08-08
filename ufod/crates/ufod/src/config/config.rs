//! This module contains Copperd's config defaults & deserializer.
//! A few notes:
//!
//! - All config fields that *can* have a default *should* have a default
//! - All config fields should be listed and documented in `default-config.toml`

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{
	error::Error,
	fmt::Display,
	fs::File,
	io::Write,
	path::{Path, PathBuf},
	time::Duration,
};

/// Server configuration
#[derive(Deserialize, Debug)]
pub struct UfodConfig {
	/// Network settings
	pub network: UfodNetworkConfig,

	/// Path settings
	pub paths: UfodPathConfig,

	#[serde(default)]
	pub pipeline: UfodPipelineConfig,

	#[serde(default)]
	pub upload: UfodUploadConfig,

	#[serde(default)]
	pub logging: UfodLoggingConfig,
}

impl UfodConfig {
	// TODO: build script to make sure this is valid
	const DEFAULT_CONFIG: &'static str = include_str!("./default-config.toml");

	/// Write the default config to the given path, overwriting if it already exists.
	pub fn create_default_config(path: &Path) -> Result<(), std::io::Error> {
		let mut file = File::create(path)?;
		file.write_all(Self::DEFAULT_CONFIG.as_bytes())?;
		return Ok(());
	}

	/// Load a config from a file.
	///
	/// This is the only valid way to make a UfodConfig,
	/// since this method makes sure paths are valid
	pub fn load_from_file(config_path: &Path) -> Result<Self, Box<dyn Error>> {
		let config_path = std::fs::canonicalize(config_path)?;
		let config_string = std::fs::read_to_string(&config_path)?;
		let mut config: Self = toml::from_str(&config_string)?;

		// Now, adjust paths so that they are relative to the config file
		config.paths.set_relative_to(config_path.parent().unwrap());
		return Ok(config);
	}
}

/// Pipeline runner settings
#[derive(Deserialize, Debug)]
pub struct UfodPipelineConfig {
	/// The maximum size, in bytes, of a binary fragment in the pipeline.
	/// Smaller values slow down pipelines; larger values use more memory.
	#[serde(default = "UfodPipelineConfig::default_frag_size")]
	pub blob_fragment_size: u64,

	/// How many pipeline jobs to run at once
	#[serde(default = "UfodPipelineConfig::default_parallel_jobs")]
	pub parallel_jobs: usize,

	/// How many threads each job may use
	#[serde(default = "UfodPipelineConfig::default_job_threads")]
	pub threads_per_job: usize,
}

impl Default for UfodPipelineConfig {
	fn default() -> Self {
		Self {
			blob_fragment_size: Self::default_frag_size(),
			parallel_jobs: Self::default_parallel_jobs(),
			threads_per_job: Self::default_job_threads(),
		}
	}
}

impl UfodPipelineConfig {
	fn default_frag_size() -> u64 {
		2_000_000
	}

	fn default_parallel_jobs() -> usize {
		4
	}

	fn default_job_threads() -> usize {
		4
	}
}

/// Network settings
#[derive(Deserialize, Debug)]
pub struct UfodNetworkConfig {
	/// IP and port to bind to
	/// Should look like `127.0.0.1:3030`
	pub server_addr: SmartString<LazyCompact>,

	// TODO: deserialize from pretty string like "2MB"
	/// Maximum request body size, in bytes
	/// If you're using a reverse proxy, make sure it
	/// also accepts requests of this size!
	pub request_body_limit: usize,
}

#[derive(Deserialize, Debug)]
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

/// Logging settings
#[derive(Deserialize, Debug, Default)]
pub struct UfodLoggingConfig {
	#[serde(default)]
	pub level: UfodLogLevelConfig,
}

/// Logging settings
#[derive(Deserialize, Debug)]
pub struct UfodLogLevelConfig {
	#[serde(default)]
	pub sqlx: LogLevel,

	#[serde(default)]
	pub http: LogLevel,

	#[serde(default)]
	pub pipeline: LogLevel,

	#[serde(default)]
	pub dataset: LogLevel,

	#[serde(default)]
	pub server: LogLevel,

	#[serde(default = "UfodLogLevelConfig::default_all")]
	pub all: LogLevel,
}

impl Default for UfodLogLevelConfig {
	fn default() -> Self {
		Self {
			sqlx: LogLevel::default(),
			http: LogLevel::default(),
			pipeline: LogLevel::default(),
			server: LogLevel::default(),
			dataset: LogLevel::default(),

			// This can get noisy, so default to a higher level
			all: Self::default_all(),
		}
	}
}

impl UfodLogLevelConfig {
	fn default_all() -> LogLevel {
		LogLevel::Warn
	}

	/// Convert this logging config to a tracing env filter
	pub fn to_env_filter(&self) -> String {
		format!(
			"ufo_pipeline={},sqlx={},tower_http={},ufod={},ufo_ds_impl={},{}",
			self.pipeline, self.sqlx, self.http, self.server, self.dataset, self.all
		)
	}
}

/// Path settings
#[derive(Deserialize, Debug)]
pub struct UfodPathConfig {
	/// Directory for in-progress uploads
	pub upload_dir: PathBuf,

	/// Where to store datasets
	pub dataset_dir: PathBuf,

	/// Main server database file (sqlite)
	pub main_db: PathBuf,
}

impl UfodPathConfig {
	/// Adjust all paths in this config to be relative to `root_path`
	fn set_relative_to(&mut self, root_path: &Path) {
		self.upload_dir = root_path.join(&self.upload_dir);
		self.dataset_dir = root_path.join(&self.dataset_dir);
		self.main_db = root_path.join(&self.main_db);
	}
}

/// Uploader settings
#[derive(Deserialize, Debug)]
pub struct UfodUploadConfig {
	/// Delete upload jobs that have been bound to a pipeline
	/// after this many seconds of inactivity
	#[serde(default = "UfodUploadConfig::default_job_timeout_bound")]
	pub job_timeout_bound: Duration,

	/// Delete unbound upload jobs after this many seconds of inacivity
	#[serde(default = "UfodUploadConfig::default_job_timeout_unbound")]
	pub job_timeout_unbound: Duration,
}

impl Default for UfodUploadConfig {
	fn default() -> Self {
		Self {
			job_timeout_bound: Self::default_job_timeout_bound(),
			job_timeout_unbound: Self::default_job_timeout_unbound(),
		}
	}
}

impl UfodUploadConfig {
	fn default_job_timeout_bound() -> Duration {
		Duration::from_secs(10)
	}

	fn default_job_timeout_unbound() -> Duration {
		Duration::from_secs(60)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Make sure the default config we ship with is valid
	#[test]
	fn default_config_is_valid() {
		let _x: UfodConfig = toml::from_str(UfodConfig::DEFAULT_CONFIG).unwrap();
	}
}
