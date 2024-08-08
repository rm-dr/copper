use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection, SqlitePool};
use std::{path::Path, str::FromStr, sync::Arc};
use tracing::{error, info};

use crate::config::CopperConfig;

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
			.bind("copper_version")
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

	pub async fn open(config: Arc<CopperConfig>) -> Result<Self, sqlx::Error> {
		let db_addr = format!("sqlite:{}?mode=rw", config.paths.main_db.to_str().unwrap());
		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::from_str(&db_addr)?
				.statement_cache_capacity(100)
				.synchronous(sqlx::sqlite::SqliteSynchronous::Extra),
		)
		.await?;

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
			auth: AuthProvider::new(pool.clone()),
			dataset: DatasetProvider::new(pool.clone(), config),
		})
	}
}
