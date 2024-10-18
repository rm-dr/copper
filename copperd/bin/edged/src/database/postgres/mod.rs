use copper_migrate::{MigrationError, Migrator};
use sqlx::{
	postgres::{PgConnection, PgPoolOptions},
	Connection, PgPool,
};
use std::{error::Error, fmt::Display};
use tracing::info;

mod client;
mod migrate;

#[derive(Debug)]
/// An error we may encounter when connecting to postgres
pub enum PgDatabaseOpenError {
	/// We encountered an internal database error
	Database(sqlx::Error),

	/// We encountered an error while migrating
	Migrate(MigrationError),
}

impl Display for PgDatabaseOpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Database(_) => write!(f, "sql error"),
			Self::Migrate(_) => write!(f, "migration error"),
		}
	}
}

impl Error for PgDatabaseOpenError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Database(e) => Some(e),
			Self::Migrate(e) => Some(e),
		}
	}
}

impl From<sqlx::Error> for PgDatabaseOpenError {
	fn from(value: sqlx::Error) -> Self {
		Self::Database(value)
	}
}

impl From<MigrationError> for PgDatabaseOpenError {
	fn from(value: MigrationError) -> Self {
		Self::Migrate(value)
	}
}

/// A database client for Postgres
pub struct PgDatabaseClient {
	pool: PgPool,
}

impl PgDatabaseClient {
	/// Create a new [`LocalDataset`].
	pub async fn open(db_addr: &str) -> Result<Self, PgDatabaseOpenError> {
		info!(message = "Opening dataset", ds_type = "postgres", ?db_addr);

		// Apply migrations
		let mut conn = PgConnection::connect(db_addr).await?;
		let mut mig = Migrator::new(&mut conn, db_addr, migrate::MIGRATE_STEPS).await?;
		mig.up().await.map_err(PgDatabaseOpenError::Migrate)?;

		drop(mig);
		drop(conn);

		let pool = PgPoolOptions::new()
			// TODO: configure
			.max_connections(5)
			.connect(db_addr)
			.await?;

		Ok(Self { pool })
	}
}
