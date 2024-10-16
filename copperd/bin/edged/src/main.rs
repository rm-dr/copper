use api::RouterState;
use auth::AuthHelper;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use axum::Router;
use config::EdgedConfig;
use copper_pipelined::client::ReqwestPipelineClient;
use copper_storaged::client::ReqwestStoragedClient;
use copper_util::{load_env, s3client::S3Client};
use database::postgres::{PgDatabaseClient, PgDatabaseOpenError};
use std::sync::Arc;
use tracing::{debug, error, info};
use uploader::Uploader;

mod api;
mod config;
mod database;

mod auth;
mod uploader;

async fn make_app(config: Arc<EdgedConfig>, objectstore_client: Arc<S3Client>) -> Router {
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
		uploader: Arc::new(Uploader::new(config.clone(), objectstore_client.clone())),

		pipelined_client: Arc::new(
			ReqwestPipelineClient::new(
				&config.edged_pipelined_addr,
				&config.edged_pipelined_secret,
			)
			// TODO: handle error
			.unwrap(),
		),

		storaged_client: Arc::new(
			ReqwestStoragedClient::new(&config.edged_storaged_addr, &config.edged_storaged_secret)
				// TODO: handle error
				.unwrap(),
		),

		objectstore_client,
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

	let cred = Credentials::new(
		&config.edged_objectstore_key_id,
		&config.edged_objectstore_key_secret,
		None,
		None,
		"pipelined .env",
	);

	// Config for minio
	let s3_config = aws_sdk_s3::config::Builder::new()
		.behavior_version(BehaviorVersion::v2024_03_28())
		.endpoint_url(&config.edged_objectstore_url)
		.credentials_provider(cred)
		.force_path_style(true)
		.region(Region::new("us-west"))
		.build();

	let client = aws_sdk_s3::Client::from_conf(s3_config);

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

	let app = make_app(
		config.clone(),
		Arc::new(S3Client::new(client.clone(), &config.edged_objectstore_bucket).await),
	)
	.await;

	match axum::serve(listener, app).await {
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", error = ?e)
		}
	};
}
