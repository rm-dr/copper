use copper_migrate::{MigrationError, Migrator};
use sqlx::{
	postgres::{PgConnection, PgPoolOptions},
	Connection, PgPool,
};
use thiserror::Error;
use tracing::info;

mod client;
mod migrate;

#[derive(Debug, Error)]
/// An error we may encounter when connecting to postgres
pub enum PgDatabaseOpenError {
	/// We encountered an internal database error
	#[error("sql error")]
	Database(#[from] sqlx::Error),

	/// We encountered an error while migrating
	#[error("migration error")]
	Migrate(#[from] MigrationError),
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
