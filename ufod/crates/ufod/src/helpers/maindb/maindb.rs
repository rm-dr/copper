use smartstring::{LazyCompact, SmartString};
use sqlx::{Connection, SqliteConnection};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};
use ufo_ds_impl::local::LocalDataset;

use crate::config::UfodConfig;

pub struct MainDB {
	pub(super) conn: Mutex<SqliteConnection>,
	pub(super) config: Arc<UfodConfig>,

	pub(super) open_datasets: Mutex<Vec<(SmartString<LazyCompact>, Arc<LocalDataset>)>>,
}

impl MainDB {
	pub async fn create(db_path: &Path) -> Result<(), sqlx::Error> {
		let db_addr = format!("sqlite:{}?mode=rwc", db_path.to_str().unwrap());
		let mut conn = SqliteConnection::connect(&db_addr).await?;

		sqlx::query(include_str!("./init.sql"))
			.execute(&mut conn)
			.await?;

		sqlx::query("INSERT INTO meta (var, val) VALUES (?, ?);")
			.bind("ufo_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut conn)
			.await?;

		Ok(())
	}

	pub async fn open(config: Arc<UfodConfig>) -> Result<Self, sqlx::Error> {
		let db_addr = format!("sqlite:{}?mode=rw", config.paths.main_db.to_str().unwrap());
		let conn = SqliteConnection::connect(&db_addr).await?;

		// Initialize dataset dir
		if !config.paths.dataset_dir.exists() {
			info!(
				message = "Creating dataset dir because it doesn't exist",
				dataset_dir = ?config.paths.dataset_dir
			);
			std::fs::create_dir_all(&config.paths.dataset_dir).unwrap();
		} else if !config.paths.dataset_dir.is_dir() {
			error!(
				message = "Dataset dir is not a directory",
				dataset_dir = ?config.paths.dataset_dir
			);
			panic!(
				"Dataset dir {:?} is not a directory",
				config.paths.dataset_dir
			)
		}

		Ok(Self {
			conn: Mutex::new(conn),
			config,
			open_datasets: Mutex::new(Vec::new()),
		})
	}
}
