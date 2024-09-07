//! Copper's storage daemon
//!
//! This daemon stores datasets and wraps all operations on them.
//! TODO: more details

use api::RouterState;
use axum::Router;
use config::StoragedConfig;
use copper_database::sqlite::{SqliteDatabaseClient, SqliteDatabaseOpenError};
use std::sync::Arc;
use tracing::{error, info};

mod api;
mod config;

async fn make_app(config: StoragedConfig) -> Router {
	// Connect to database
	let db = match SqliteDatabaseClient::open(&config.db_addr).await {
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

	// Create app
	return api::router(RouterState {
		config: Arc::new(config.clone()),
		client: Arc::new(db),
	});
}

#[tokio::main]
async fn main() {
	// TODO: configure with env vars
	let config = StoragedConfig::default();

	tracing_subscriber::fmt()
		.with_env_filter(config.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	let listener = match tokio::net::TcpListener::bind(config.server_addr.to_string()).await {
		Ok(x) => x,
		Err(e) => {
			match e.kind() {
				std::io::ErrorKind::AddrInUse => {
					error!(
						message = "Cannot bind to address, already in use",
						server_addr = config.server_addr.as_str()
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

	let app = make_app(config).await;

	match axum::serve(listener, app).await {
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", error = ?e)
		}
	};
}

#[cfg(test)]
mod tests {
	use std::{path::PathBuf, usize};

	use super::*;
	use axum::{
		body::Body,
		http::{Method, Request, Response},
	};
	use serde::{de::DeserializeOwned, Deserialize};
	use serde_json::json;
	use tower::Service;

	async fn app_request(
		app: &mut Router,
		method: Method,
		url: &str,
		body: serde_json::Value,
	) -> Response<Body> {
		app.call(
			Request::builder()
				.method(method)
				.header(axum::http::header::CONTENT_TYPE, "application/json")
				.uri(url)
				.body(Body::from(serde_json::to_string(&body).unwrap()))
				.unwrap(),
		)
		.await
		.unwrap()
	}

	async fn create_dataset(app: &mut Router, name: &str) -> Response<Body> {
		app_request(
			app,
			Method::POST,
			"/dataset",
			json!({
				"name": name
			}),
		)
		.await
	}

	async fn response_json<T: DeserializeOwned>(resp: Response<Body>) -> T {
		serde_json::from_str(
			&String::from_utf8(
				axum::body::to_bytes(resp.into_body(), usize::MAX)
					.await
					.unwrap()
					.to_vec(),
			)
			.unwrap(),
		)
		.unwrap()
	}

	#[tokio::test]
	async fn basic_crud_sqlite() {
		// We need to use a file, since in-memory sqlite
		// misbehaves with sqlx connection pools.
		const SQLITE_DB_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_db.sqlite");

		// Delete test db if it exists
		let file_path = PathBuf::from(SQLITE_DB_FILE);
		if file_path.exists() {
			std::fs::remove_file(file_path).unwrap();
		}

		tracing_subscriber::fmt()
			.without_time()
			.with_ansi(true)
			.init();

		// Set up config & create app
		let mut config = StoragedConfig::default();
		config.db_addr = format!("sqlite://{SQLITE_DB_FILE}?mode=rwc").into();
		let mut app = make_app(config).await;

		//
		// MARK: Create datasets
		//

		{
			// These requests should fail, invalid name
			let response = create_dataset(&mut app, "").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot be empty"
			);

			let response = create_dataset(&mut app, "  test_dataset").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);

			let response = create_dataset(&mut app, "test_dataset  ").await;
			assert_eq!(response.status(), 400);
			assert_eq!(
				response_json::<String>(response).await,
				"name cannot have leading or trailing whitespace"
			);
		}

		{
			// This request is perfectly fine
			let response = create_dataset(&mut app, "test_dataset").await;
			assert_eq!(response.status(), 200);
		}

		{
			// This request should fail, duplicate name
			let response = create_dataset(&mut app, "test_dataset").await;
			assert_eq!(response.status(), 400);
		}
	}
}
