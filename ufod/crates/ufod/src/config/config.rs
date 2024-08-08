use std::{
	error::Error,
	fmt::Display,
	fs::File,
	io::{Read, Write},
	path::{Path, PathBuf},
	time::Duration,
};

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};

/// Ufod server configuration
#[derive(Deserialize, Debug)]
pub struct UfodConfig {
	/// Network settings
	pub network: UfodNetworkConfig,

	/// Path settings
	pub paths: UfodPathConfig,

	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: u64,

	/// Uploader settings
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

		let mut f = File::open(&config_path)?;
		let mut config_string = String::new();
		f.read_to_string(&mut config_string)?;
		let mut config: Self = toml::from_str(&config_string)?;

		// Now, adjust paths so that they are relative to the config file
		config.paths.set_relative_to(config_path.parent().unwrap());
		return Ok(config);
	}
}

/// Ufod network settings
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

/// Ufod logging settings
#[derive(Deserialize, Debug)]
pub struct UfodLoggingConfig {
	#[serde(default)]
	pub sqlx: LogLevel,

	#[serde(default)]
	pub http: LogLevel,

	#[serde(default)]
	pub pipeline: LogLevel,

	#[serde(default = "UfodLoggingConfig::default_warn")]
	pub all: LogLevel,
}

impl Default for UfodLoggingConfig {
	fn default() -> Self {
		Self {
			sqlx: Default::default(),
			http: Default::default(),
			pipeline: Default::default(),

			// This can get noisy, so default to a higher level
			all: LogLevel::Warn,
		}
	}
}

impl UfodLoggingConfig {
	fn default_warn() -> LogLevel {
		LogLevel::Warn
	}

	/// Convert this logging config to a tracing env filter
	pub fn to_env_filter(&self) -> String {
		format!(
			"ufo_pipeline={},sqlx={},tower_http={},{}",
			self.pipeline, self.sqlx, self.http, self.all
		)
	}
}

/// Ufod path settings
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

/// Ufod uploader settings
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
