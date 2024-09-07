//! Simple database migration utility

#![warn(missing_docs)]

use std::{collections::BTreeMap, error::Error, fmt::Display};

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use sqlx::{Row, SqliteConnection};
use time::OffsetDateTime;
use tracing::{debug, info};

/// One step in a database migration
#[async_trait::async_trait]
pub trait Migration {
	/// This migration's name
	fn name(&self) -> &str;

	/// Apply this migration
	async fn up(&self, conn: &mut SqliteConnection) -> Result<(), sqlx::Error>;

	/// Unto this migration
	async fn down(&self, _conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
		// Unused right now
		unimplemented!()
	}
}

/// An error we encounter while migrating
#[derive(Debug)]
pub enum MigrationError {
	/// An sql query resulted in an error
	DbError(sqlx::Error),

	/// The migrations already applied on a database did not match
	/// those we expected.
	BadExistingMigrations,

	/// We could not deserialize a migration record
	MalformedMigrationRecord,
}

impl Display for MigrationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "sql error while migrating"),
			Self::BadExistingMigrations => write!(f, "bad existing migrations"),
			Self::MalformedMigrationRecord => write!(f, "could not deserialize migration"),
		}
	}
}

impl Error for MigrationError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(e) => Some(e),
			_ => None,
		}
	}
}

impl From<sqlx::Error> for MigrationError {
	fn from(value: sqlx::Error) -> Self {
		Self::DbError(value)
	}
}

/// A migration entry in the database,
/// recording a migration that has already been applied
#[derive(Serialize, Deserialize, Debug)]
struct MigrationRecord {
	/// This migration's name
	name: SmartString<LazyCompact>,

	/// The time this migration was applied
	applied_at: OffsetDateTime,
}

struct MigrationStatus<'a> {
	applied: bool,
	migration: &'a dyn Migration,
}

/// A helper struct that applies migrations to a database
pub struct Migrator<'a> {
	/// The steps in this migration, in the order they're run.
	///
	/// The first migration in this array is the first migration that is run.
	/// It is always executed on a freshly-created database.
	migrations: Vec<MigrationStatus<'a>>,

	/// A connection to the database we're migrating
	conn: &'a mut SqliteConnection,

	/// The location of the database we're migrating, used only for debug
	name_of_db: String,
}

impl<'a> Migrator<'a> {
	/// Create a new migration with the given steps
	pub async fn new(
		conn: &'a mut SqliteConnection,
		name_of_db: &str,
		migrations: &'a [&'a dyn Migration],
	) -> Result<Self, MigrationError> {
		// Initialize migration table
		sqlx::query(
			"
			CREATE TABLE IF NOT EXISTS copper_migrations (
				var TEXT NOT NULL,
				val TEXT NOT NULL
			);
			",
		)
		.execute(&mut *conn)
		.await?;

		// Get applied migrations
		let res = sqlx::query(
			"
			SELECT * FROM copper_migrations
			WHERE var=\"migration\";
			",
		)
		.fetch_all(&mut *conn)
		.await?;

		let mut ap_migs: BTreeMap<SmartString<LazyCompact>, MigrationRecord> = BTreeMap::new();
		for row in res {
			let r: MigrationRecord = serde_json::from_str(row.get("val"))
				.map_err(|_| MigrationError::MalformedMigrationRecord)?;
			ap_migs.insert(r.name.clone(), r);
		}

		let mut entered_new_migrations = false;
		let mut migs = Vec::new();
		for m in migrations {
			let applied = ap_migs.remove(m.name()).is_some();
			migs.push(MigrationStatus {
				applied,
				migration: *m,
			});

			// If we encounter one non-applied migration,
			// all later migrations must not be applied.
			if !applied {
				entered_new_migrations = true;
			} else if entered_new_migrations {
				return Err(MigrationError::BadExistingMigrations);
			}
		}

		// If this is not zero, there is an applied migration we did not expect.
		if !ap_migs.is_empty() {
			return Err(MigrationError::BadExistingMigrations);
		}

		return Ok(Self {
			conn,
			migrations: migs,
			name_of_db: name_of_db.into(),
		});
	}

	/// Apply all migrations that have not yet been run on this database.
	pub async fn up(&mut self) -> Result<(), MigrationError> {
		let mut entered_new_migrations = false;
		for mig in &mut self.migrations {
			if mig.applied {
				debug!(
					message = "Skipping migration, already applied",
					migration = mig.migration.name(),
					database = self.name_of_db
				);

				if entered_new_migrations {
					return Err(MigrationError::BadExistingMigrations);
				}

				continue;
			}

			if !entered_new_migrations {
				entered_new_migrations = true;
			}

			info!(
				message = "Applying migration",
				migration = mig.migration.name(),
				database = self.name_of_db
			);

			mig.migration.up(self.conn).await?;
			mig.applied = true;

			sqlx::query("INSERT INTO copper_migrations (var, val) VALUES (?, ?);")
				.bind("migration")
				.bind(
					serde_json::to_string(&MigrationRecord {
						name: mig.migration.name().into(),
						applied_at: OffsetDateTime::now_utc(),
					})
					.unwrap(),
				)
				.execute(&mut *self.conn)
				.await?;
		}

		return Ok(());
	}
}
