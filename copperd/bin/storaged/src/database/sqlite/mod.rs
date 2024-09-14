//! A database client for SQLite

use copper_migrate::{MigrationError, Migrator};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection, SqlitePool};
use std::{error::Error, fmt::Display, str::FromStr};
use tracing::info;

mod client;
mod helpers;
mod migrate;

#[derive(Debug)]
/// An error we encounter when opening an SQLite database
pub enum SqliteDatabaseOpenError {
	/// We encountered an internal database error
	Database(Box<dyn Error + Send + Sync>),

	/// We encountered an error while migrating
	Migrate(MigrationError),
}

impl Display for SqliteDatabaseOpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Database(_) => write!(f, "sql error"),
			Self::Migrate(_) => write!(f, "migration error"),
		}
	}
}

impl Error for SqliteDatabaseOpenError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Database(e) => Some(e.as_ref()),
			Self::Migrate(e) => Some(e),
		}
	}
}

/// A database client for SQLite
pub struct SqliteDatabaseClient {
	pool: SqlitePool,
}

impl SqliteDatabaseClient {
	/// Create a new [`LocalDataset`].
	pub async fn open(db_addr: &str) -> Result<Self, SqliteDatabaseOpenError> {
		info!(message = "Creating dataset", ds_type = "sqlite", ?db_addr);

		// Apply migrations
		let mut conn = SqliteConnection::connect(db_addr)
			.await
			.map_err(|e| SqliteDatabaseOpenError::Database(Box::new(e)))?;
		let mut mig = Migrator::new(&mut conn, db_addr, migrate::MIGRATE_STEPS)
			.await
			.map_err(|x| SqliteDatabaseOpenError::Database(Box::new(x)))?;
		mig.up().await.map_err(SqliteDatabaseOpenError::Migrate)?;

		drop(mig);
		drop(conn);

		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::from_str(db_addr)
				.map_err(|e| SqliteDatabaseOpenError::Database(Box::new(e)))?
				// Disable statement cache. Each connection in this pool will have its own statement cache,
				// so the cache-clearing we do in the code below won't clear all statement caches.
				.statement_cache_capacity(0)
				.synchronous(sqlx::sqlite::SqliteSynchronous::Extra),
		)
		.await
		.map_err(|e| SqliteDatabaseOpenError::Database(Box::new(e)))?;

		Ok(Self { pool })
	}
}
