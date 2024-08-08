use sqlx::{Connection, SqliteConnection};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::config::UfodConfig;

use super::{auth::AuthProvider, dataset::DatasetProvider};

pub struct MainDB {
	pub auth: AuthProvider,
	pub dataset: DatasetProvider,
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

		// Add default admin account
		sqlx::query("INSERT INTO users (user_name, pw_hash, user_group) VALUES (?, ?, ?);")
			.bind("admin")
			.bind(AuthProvider::hash_password("admin"))
			.bind(None::<u32>)
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

		let conn = Arc::new(Mutex::new(conn));

		Ok(Self {
			auth: AuthProvider::new(conn.clone()),
			dataset: DatasetProvider::new(conn.clone(), config),
		})
	}
}
