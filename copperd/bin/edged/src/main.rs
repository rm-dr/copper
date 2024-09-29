use api::RouterState;
use auth::AuthHelper;
use axum::Router;
use config::EdgedConfig;
use copper_storaged::client::ReqwestStoragedClient;
use copper_util::load_env;
use database::postgres::{PgDatabaseClient, PgDatabaseOpenError};
use std::sync::Arc;
use tracing::{debug, error, info};

mod api;
mod auth;
mod config;
mod database;

async fn make_app(config: Arc<EdgedConfig>) -> Router {
	// Connect to database
	let db = match PgDatabaseClient::open(&config.edged_db_addr).await {
		Ok(db) => db,
		Err(PgDatabaseOpenError::Database(e)) => {
			error!(message = "SQL error while opening database", err = ?e);
			std::process::exit(1);
		}
		Err(PgDatabaseOpenError::Migrate(e)) => {
			error!(message = "Migration error while opening database", err = ?e);
			std::process::exit(1);
		}
	};

	// Create app
	return api::router(RouterState {
		config: config.clone(),
		db_client: Arc::new(db),
		auth: Arc::new(AuthHelper::new()),
		storaged_client: Arc::new(
			ReqwestStoragedClient::new(
				config.edged_storaged_addr.clone(),
				&config.edged_storaged_secret,
			)
			// TODO: handle error
			.unwrap(),
		),
	});
}

#[tokio::main]
async fn main() {
	let config = Arc::new(load_env::<EdgedConfig>());

	tracing_subscriber::fmt()
		.with_env_filter(config.edged_loglevel.get_config())
		.without_time()
		.with_ansi(true)
		.init();

	debug!(message = "Loaded config from environment", ?config);

	let listener = match tokio::net::TcpListener::bind(config.edged_server_addr.to_string()).await {
		Ok(x) => x,
		Err(e) => {
			match e.kind() {
				std::io::ErrorKind::AddrInUse => {
					error!(
						message = "Cannot bind to address, already in use",
						server_addr = config.edged_server_addr.as_str()
					);
				}
				_ => {
					error!(message = "Error while migrating database", err = ?e);
				}
			}

			std::process::exit(1);
		}
	};
	info!("listening on http://{}", listener.local_addr().unwrap());

	let app = make_app(config).await;

	match axum::serve(listener, app).await {
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", error = ?e)
		}
	};
}
