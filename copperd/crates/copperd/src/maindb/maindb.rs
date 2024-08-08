use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection, SqlitePool};
use std::{error::Error, fmt::Display, path::Path, str::FromStr, sync::Arc};
use tracing::{error, info};

use crate::config::CopperConfig;

use super::{auth::AuthProvider, dataset::DatasetProvider};

#[derive(Debug)]
pub enum MainDbCreateError {
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for MainDbCreateError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "sql error"),
		}
	}
}

impl Error for MainDbCreateError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(e) => Some(e.as_ref()),
		}
	}
}

#[derive(Debug)]
pub enum MainDbOpenError {
	DbError(Box<dyn Error + Send + Sync>),
	IoError(std::io::Error),
	BadDatasetDir,
}

impl Display for MainDbOpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "sql error"),
			Self::IoError(_) => write!(f, "i/o error"),
			Self::BadDatasetDir => write!(f, "this path is not a directory"),
		}
	}
}

impl Error for MainDbOpenError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(e) => Some(e.as_ref()),
			Self::IoError(e) => Some(e),
			_ => return None,
		}
	}
}

pub struct MainDB {
	pub auth: AuthProvider,
	pub dataset: DatasetProvider,
}

impl MainDB {
	pub async fn create(db_path: &Path) -> Result<(), MainDbCreateError> {
		let db_addr = format!("sqlite:{}?mode=rwc", db_path.to_str().unwrap());

		let mut conn = SqliteConnection::connect(&db_addr)
			.await
			.map_err(|e| MainDbCreateError::DbError(Box::new(e)))?;

		sqlx::query(include_str!("./init.sql"))
			.execute(&mut conn)
			.await
			.map_err(|e| MainDbCreateError::DbError(Box::new(e)))?;

		sqlx::query("INSERT INTO meta (var, val) VALUES (?, ?);")
			.bind("copper_version")
			.bind(env!("CARGO_PKG_VERSION"))
			.execute(&mut conn)
			.await
			.map_err(|e| MainDbCreateError::DbError(Box::new(e)))?;

		// Add default admin account
		sqlx::query("INSERT INTO users (user_name, pw_hash, user_group) VALUES (?, ?, ?);")
			.bind("admin")
			.bind(AuthProvider::hash_password("admin"))
			.bind(None::<u32>)
			.execute(&mut conn)
			.await
			.map_err(|e| MainDbCreateError::DbError(Box::new(e)))?;

		Ok(())
	}

	pub async fn open(config: Arc<CopperConfig>) -> Result<Self, MainDbOpenError> {
		let db_addr = format!("sqlite:{}?mode=rw", config.paths.main_db.to_str().unwrap());
		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::from_str(&db_addr)
				.map_err(|e| MainDbOpenError::DbError(Box::new(e)))?
				.statement_cache_capacity(100)
				.synchronous(sqlx::sqlite::SqliteSynchronous::Extra),
		)
		.await
		.map_err(|e| MainDbOpenError::DbError(Box::new(e)))?;

		// Initialize dataset dir
		if !config.paths.dataset_dir.exists() {
			info!(
				message = "Creating dataset dir because it doesn't exist",
				dataset_dir = ?config.paths.dataset_dir
			);
			std::fs::create_dir_all(&config.paths.dataset_dir).map_err(MainDbOpenError::IoError)?;
		} else if !config.paths.dataset_dir.is_dir() {
			error!(
				message = "Dataset dir is not a directory",
				dataset_dir = ?config.paths.dataset_dir
			);
			return Err(MainDbOpenError::BadDatasetDir);
		}

		Ok(Self {
			auth: AuthProvider::new(pool.clone()),
			dataset: DatasetProvider::new(pool.clone(), config),
		})
	}
}