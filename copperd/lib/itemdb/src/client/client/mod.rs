//! This modules contains Copper's itemdb client

use copper_migrate::{MigrationError, Migrator};
use sqlx::{
	pool::PoolConnection, postgres::PgPoolOptions, Connection, PgConnection, PgPool, Postgres,
};
use thiserror::Error;
use tracing::info;

use crate::client::migrate;

mod attribute;
mod class;
mod dataset;
mod item;

#[derive(Debug, Error)]
/// An error we may encounter when connecting to postgres
pub enum ItemdbOpenError {
	/// We encountered an internal database error
	#[error("sql error")]
	Database(#[from] sqlx::Error),

	/// We encountered an error while migrating
	#[error("migration error")]
	Migrate(#[from] MigrationError),

	/// We opened a database with `migrate = false`,
	/// but this database has not been migrated.
	#[error("database not migrated")]
	NotMigrated,
}

/// A database client for postgres
pub struct ItemdbClient {
	pool: PgPool,
}

impl ItemdbClient {
	/// Create a new [`LocalDataset`].
	pub async fn open(db_addr: &str, migrate: bool) -> Result<Self, ItemdbOpenError> {
		info!(message = "Opening dataset", ds_type = "postgres", ?db_addr);

		// Apply migrations
		let mut conn = PgConnection::connect(db_addr).await?;
		let mut mig = Migrator::new(&mut conn, db_addr, migrate::MIGRATE_STEPS).await?;

		if migrate {
			mig.up().await.map_err(ItemdbOpenError::Migrate)?;
		} else if !mig.is_up()? {
			return Err(ItemdbOpenError::NotMigrated);
		}

		drop(mig);
		drop(conn);

		let pool = PgPoolOptions::new()
			// TODO: configure
			.max_connections(5)
			.connect(db_addr)
			.await?;

		Ok(Self { pool })
	}

	pub async fn new_connection(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
		let conn = self.pool.acquire().await?;
		return Ok(conn);
	}
}
