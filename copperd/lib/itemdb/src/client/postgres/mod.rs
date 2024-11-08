use copper_migrate::{MigrationError, Migrator};
use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool};
use std::{error::Error, fmt::Display};
use tracing::info;

mod client;
mod helpers;
mod migrate;

#[derive(Debug)]
/// An error we may encounter when connecting to postgres
pub enum PgItemdbOpenError {
	/// We encountered an internal database error
	Database(sqlx::Error),

	/// We encountered an error while migrating
	Migrate(MigrationError),

	/// We opened a database with `migrate = false`,
	/// but this database has not been migrated.
	NotMigrated,
}

impl Display for PgItemdbOpenError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Database(_) => write!(f, "sql error"),
			Self::Migrate(_) => write!(f, "migration error"),
			Self::NotMigrated => write!(f, "database not migrated"),
		}
	}
}

impl Error for PgItemdbOpenError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Database(e) => Some(e),
			Self::Migrate(e) => Some(e),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for PgItemdbOpenError {
	fn from(value: sqlx::Error) -> Self {
		Self::Database(value)
	}
}

impl From<MigrationError> for PgItemdbOpenError {
	fn from(value: MigrationError) -> Self {
		Self::Migrate(value)
	}
}

/// A database client for postgres
pub struct PgItemdbClient {
	pool: PgPool,
}

impl PgItemdbClient {
	/// Create a new [`LocalDataset`].
	pub async fn open(db_addr: &str, migrate: bool) -> Result<Self, PgItemdbOpenError> {
		info!(message = "Opening dataset", ds_type = "postgres", ?db_addr);

		// Apply migrations
		let mut conn = PgConnection::connect(db_addr).await?;
		let mut mig = Migrator::new(&mut conn, db_addr, migrate::MIGRATE_STEPS).await?;

		if migrate {
			mig.up().await.map_err(PgItemdbOpenError::Migrate)?;
		} else if !mig.is_up()? {
			return Err(PgItemdbOpenError::NotMigrated);
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
}
