use std::{path::PathBuf, time::Duration};

use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};

#[derive(Deserialize, Debug, Clone)]
pub struct UfodConfig {
	/// ip and port to bind to
	pub server_addr: SmartString<LazyCompact>,

	/// Maximum body size, in bytes
	pub request_body_limit: usize,

	/// Delete upload jobs that have been bound to a pipeline
	/// after this many seconds of inactivity
	pub delete_job_after_bound: Duration,

	/// Delete unbound upload jobs after this many seconds of inacivity
	pub delete_job_after_unbound: Duration,

	/// Directory for in-progress uploads
	pub upload_dir: PathBuf,

	/// Where to store datasets
	pub dataset_dir: PathBuf,

	/// Main server db location
	pub main_db: PathBuf,
}

impl Default for UfodConfig {
	fn default() -> Self {
		Self {
			server_addr: "127.0.0.1:3030".into(),
			request_body_limit: 2 * 1024 * 1024, // 2Mb
			delete_job_after_bound: Duration::from_secs(5),
			delete_job_after_unbound: Duration::from_secs(10),
			upload_dir: "./data/tmp".into(),
			dataset_dir: "./data/datasets".into(),
			main_db: "./data/ufo.sqlite".into(),
		}
	}
}
