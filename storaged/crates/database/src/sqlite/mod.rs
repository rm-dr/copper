use copper_migrate::{MigrationError, Migrator};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection, SqlitePool};
use std::{error::Error, fmt::Display, str::FromStr};
use tracing::info;

mod meta;
mod migrate;

#[derive(Debug)]
pub enum SqliteDatabaseOpenError {
	DbError(Box<dyn Error + Send + Sync>),
	IoError(std::io::Error),
	MigrateError(MigrationError),
}

impl Display for SqliteDatabaseOpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "sql error"),
			Self::IoError(_) => write!(f, "i/o error"),
			Self::MigrateError(_) => write!(f, "migration error"),
		}
	}
}

impl Error for SqliteDatabaseOpenError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(e) => Some(e.as_ref()),
			Self::IoError(e) => Some(e),
			Self::MigrateError(e) => Some(e),
		}
	}
}

pub struct SqliteDatabase {
	pool: SqlitePool,
}

impl SqliteDatabase {
	/// Create a new [`LocalDataset`].
	pub async fn open(db_addr: &str) -> Result<Self, SqliteDatabaseOpenError> {
		info!(message = "Creating dataset", ds_type = "sqlite", ?db_addr);

		// Apply migrations
		let mut conn = SqliteConnection::connect(&db_addr)
			.await
			.map_err(|e| SqliteDatabaseOpenError::DbError(Box::new(e)))?;
		let mut mig = Migrator::new(&mut conn, &db_addr, migrate::MIGRATE_STEPS)
			.await
			.map_err(|x| SqliteDatabaseOpenError::DbError(Box::new(x)))?;
		mig.up()
			.await
			.map_err(SqliteDatabaseOpenError::MigrateError)?;

		drop(mig);
		drop(conn);

		let pool = SqlitePool::connect_with(
			SqliteConnectOptions::from_str(&db_addr)
				.map_err(|e| SqliteDatabaseOpenError::DbError(Box::new(e)))?
				// Disable statement cache. Each connection in this pool will have its own statement cache,
				// so the cache-clearing we do in the code below won't clear all statement caches.
				.statement_cache_capacity(0)
				.synchronous(sqlx::sqlite::SqliteSynchronous::Extra),
		)
		.await
		.map_err(|e| SqliteDatabaseOpenError::DbError(Box::new(e)))?;

		Ok(Self { pool })
	}
}
