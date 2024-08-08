use futures::executor::block_on;
use sqlx::{Connection, SqliteConnection};
use std::{path::Path, sync::Mutex};
use tracing::{error, info};

use crate::config::UfodConfig;

pub struct MainDB {
	pub(super) conn: Mutex<SqliteConnection>,
	pub(super) config: UfodConfig,
}

impl MainDB {
	pub fn create(db_path: &Path) -> Result<(), sqlx::Error> {
		let db_addr = format!("sqlite:{}?mode=rwc", db_path.to_str().unwrap());
		let mut conn = block_on(SqliteConnection::connect(&db_addr))?;

		block_on(sqlx::query(include_str!("./init.sql")).execute(&mut conn))?;
		block_on(
			sqlx::query("INSERT INTO meta (var, val) VALUES (?, ?);")
				.bind("ufo_version")
				.bind(env!("CARGO_PKG_VERSION"))
				.execute(&mut conn),
		)?;

		Ok(())
	}

	pub fn open(config: UfodConfig) -> Result<Self, sqlx::Error> {
		let db_addr = format!("sqlite:{}?mode=rw", config.main_db.to_str().unwrap());
		let conn = block_on(SqliteConnection::connect(&db_addr))?;

		// Initialize dataset dir
		if !config.dataset_dir.exists() {
			info!(
				message = "Creating dataset dir because it doesn't exist",
				dataset_dir = ?config.dataset_dir
			);
			std::fs::create_dir_all(&config.dataset_dir).unwrap();
		} else if !config.dataset_dir.is_dir() {
			error!(
				message = "Dataset dir is not a directory",
				dataset_dir = ?config.dataset_dir
			);
			panic!("Dataset dir {:?} is not a directory", config.dataset_dir)
		}

		Ok(Self {
			conn: Mutex::new(conn),
			config,
		})
	}
}
