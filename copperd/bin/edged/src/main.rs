use api::{CopperConnectInfo, RouterState};
use auth::AuthHelper;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use axum::Router;
use config::EdgedConfig;
use copper_edged::UserPassword;
use copper_jobqueue::postgres::{PgJobQueueClient, PgJobQueueOpenError};
use copper_storage::database::postgres::{PgStorageDatabaseClient, PgStorageDatabaseOpenError};
use copper_util::{load_env, s3client::S3Client, LoadedEnv};
use database::{
	base::client::DatabaseClient,
	postgres::{PgDatabaseClient, PgDatabaseOpenError},
};
use std::sync::Arc;
use tracing::{error, info, trace, warn};
use uploader::Uploader;

mod api;
mod config;
mod database;

mod apidata;
mod auth;
mod uploader;

async fn make_app(config: Arc<EdgedConfig>, s3_client_upload: Arc<S3Client>) -> Router {
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

	trace!(message = "Connecting to storage db");
	// Connect to database
	let storage_db_client =
		match PgStorageDatabaseClient::open(&config.edged_storage_db_addr, true).await {
			Ok(db) => Arc::new(db),
			Err(PgStorageDatabaseOpenError::Database(e)) => {
				error!(message = "SQL error while opening storage database", err = ?e);
				std::process::exit(1);
			}

			Err(PgStorageDatabaseOpenError::Migrate(e)) => {
				error!(message = "Migration error while opening storage database", err = ?e);
				std::process::exit(1);
			}

			Err(PgStorageDatabaseOpenError::NotMigrated) => unreachable!(),
		};
	trace!(message = "Successfully connected to storage db");

	trace!(message = "Initializing job queue client");
	let jobqueue_client = match PgJobQueueClient::open(&config.edged_jobqueue_db).await {
		Ok(db) => Arc::new(db),
		Err(PgJobQueueOpenError::Database(e)) => {
			error!(message = "SQL error while opening job queue database", err = ?e);
			std::process::exit(1);
		}
		Err(PgJobQueueOpenError::Migrate(e)) => {
			error!(message = "Migration error while opening job queue database", err = ?e);
			std::process::exit(1);
		}
	};
	trace!(message = "Successfully initialized job queue client");

	if config.edged_init_user_email.is_some() && config.edged_init_user_pass.is_some() {
		let email = config.edged_init_user_email.as_ref().unwrap();
		let pass = config.edged_init_user_pass.as_ref().unwrap();

		let user = match db.get_user_by_email(email).await {
			Ok(x) => x,
			Err(error) => {
				error!(message = "Error while checking initial user", ?error);
				std::process::exit(1);
			}
		};

		if user.is_some() {
			info!(
				message = "Not creating initial user, a user with this email already exists",
				EDGED_INIT_USER_EMAIL = config.edged_init_user_email,
				EDGED_INIT_USER_PASS = config.edged_init_user_pass
			)
		} else {
			info!(
				message = "Creating initial user",
				EDGED_INIT_USER_EMAIL = config.edged_init_user_email,
				EDGED_INIT_USER_PASS = config.edged_init_user_pass
			);
			let res = db
				.add_user(email, "Initial user", &UserPassword::new(pass))
				.await;

			match res {
				Ok(_) => {}
				Err(error) => {
					error!(message = "Error while creating initial user", ?error);
					std::process::exit(1);
				}
			};
		}
	} else if config.edged_init_user_email.is_some() || config.edged_init_user_pass.is_some() {
		warn!(
			message = "Not creating initial user, not all field were provided",
			EDGED_INIT_USER_NAME = config.edged_init_user_email,
			EDGED_INIT_USER_PASS = config.edged_init_user_pass
		)
	}

	// Create app
	return api::router(RouterState {
		config: config.clone(),
		db_client: Arc::new(db),
		auth: Arc::new(AuthHelper::new()),
		uploader: Arc::new(Uploader::new(
			config.clone(),
			s3_client_upload.clone(),
			jobqueue_client.clone(),
		)),

		jobqueue_client,
		storage_db_client,
		s3_client_upload,
	});
}

#[tokio::main]
async fn main() {
	let config_res = match load_env::<EdgedConfig>() {
		Ok(x) => x,
		Err(err) => {
			println!("Error while loading .env: {err}");
			std::process::exit(1);
		}
	};

	let config: Arc<EdgedConfig> = Arc::new(config_res.get_config().clone().validate());

	tracing_subscriber::fmt()
		.with_env_filter(config.edged_loglevel.get_config())
		.without_time()
		.with_ansi(true)
		.init();

	// Do this now, logging wasn't available earlier
	match config_res {
		LoadedEnv::FoundFile { config, path } => {
			info!(message = "Loaded config from .env", ?path, ?config);
		}
		LoadedEnv::OnlyVars(config) => {
			info!(
				message = "No `.env` found, loaded config from environment",
				?config
			);
		}
	};

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

	let client = S3Client::new(aws_sdk_s3::Client::from_conf(s3_config)).await;

	// Create upload bucket if it doesn't exist
	match client
		.create_bucket(&config.edged_objectstore_upload_bucket)
		.await
	{
		Ok(false) => {}
		Ok(true) => {
			info!(
				message = "Created upload bucket because it didn't exist",
				bucket = config.edged_objectstore_upload_bucket
			);
		}
		Err(error) => {
			error!(
				message = "Error while creating upload bucket",
				bucket = config.edged_objectstore_upload_bucket,
				?error
			);
		}
	}

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

	match listener.local_addr() {
		Ok(x) => info!("listening on http://{x}"),
		Err(error) => {
			error!(message = "Could not determine local address", ?error);
			std::process::exit(1);
		}
	}

	let app = make_app(config.clone(), Arc::new(client)).await;

	match axum::serve(
		listener,
		app.into_make_service_with_connect_info::<CopperConnectInfo>(),
	)
	.await
	{
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", error = ?e)
		}
	};
}
