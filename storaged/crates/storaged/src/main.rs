use api::RouterState;
use config::StoragedConfig;
use copper_database::sqlite::{SqliteDatabase, SqliteDatabaseOpenError};
use std::sync::Arc;
use tracing::{error, info};

mod api;
mod config;

#[tokio::main]
async fn main() {
	// TODO: configure with env vars
	let config = Arc::new(StoragedConfig::default());

	tracing_subscriber::fmt()
		.with_env_filter(config.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	// Connect to database
	let db = match SqliteDatabase::open(&config.db_addr).await {
		Ok(db) => db,
		Err(SqliteDatabaseOpenError::DbError(e)) => {
			error!(message = "SQL error while opening database", err = ?e);
			std::process::exit(1);
		}
		Err(SqliteDatabaseOpenError::MigrateError(e)) => {
			error!(message = "Migration error while opening database", err = ?e);
			std::process::exit(1);
		}
		Err(SqliteDatabaseOpenError::IoError(e)) => {
			error!(message = "I/O error while opening database", err = ?e);
			std::process::exit(1);
		}
	};

	let state = RouterState::<SqliteDatabase> {
		config,
		client: Arc::new(db),
	};

	let listener = match tokio::net::TcpListener::bind(state.config.server_addr.to_string()).await {
		Ok(x) => x,
		Err(e) => {
			match e.kind() {
				std::io::ErrorKind::AddrInUse => {
					error!(
						message = "Cannot bind to address, already in use",
						server_addr = state.config.server_addr.as_str()
					);
				}
				_ => {
					error!(message = "Error while migrating main database", err = ?e);
				}
			}

			std::process::exit(1);
		}
	};
	info!("listening on {}", listener.local_addr().unwrap());

	let app = api::router(state);

	match axum::serve(listener, app).await {
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", error = ?e)
		}
	};
}
