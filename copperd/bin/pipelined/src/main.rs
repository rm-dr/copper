use api::RouterState;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use config::{PipelinedConfig, ASYNC_POLL_AWAIT_MS};
use copper_pipelined::{data::PipeData, CopperContext};
use copper_storaged::client::ReqwestStoragedClient;
use copper_util::{load_env, s3client::S3Client, LoadedEnv};
use futures::TryFutureExt;
use pipeline::runner::{PipelineRunner, PipelineRunnerOptions};
use std::{error::Error, future::IntoFuture, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

mod api;
mod config;
mod pipeline;

// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() {
	let config_res = match load_env::<PipelinedConfig>() {
		Ok(x) => x,
		Err(err) => {
			println!("Error while loading .env: {err}");
			std::process::exit(1);
		}
	};

	let config: Arc<PipelinedConfig> = Arc::new(config_res.get_config().clone());

	tracing_subscriber::fmt()
		.with_env_filter(config.pipelined_loglevel.get_config())
		.without_time()
		.with_ansi(true)
		.init();

	// Do this now, logging wasn't available earlier
	match config_res {
		LoadedEnv::FoundFile { config, path } => {
			debug!(message = "Loaded config from .env", ?path, ?config);
		}
		LoadedEnv::OnlyVars(config) => {
			debug!(
				message = "No `.env` found, loaded config from environment",
				?config
			);
		}
	};

	let cred = Credentials::new(
		&config.pipelined_objectstore_key_id,
		&config.pipelined_objectstore_key_secret,
		None,
		None,
		"pipelined .env",
	);

	// Config for minio
	let s3_config = aws_sdk_s3::config::Builder::new()
		.behavior_version(BehaviorVersion::v2024_03_28())
		.endpoint_url(&config.pipelined_objectstore_url)
		.credentials_provider(cred)
		.region(Region::new("us-west"))
		.force_path_style(true)
		.build();

	let client = S3Client::new(aws_sdk_s3::Client::from_conf(s3_config)).await;

	// Create blobstore bucket if it doesn't exist
	match client
		.create_bucket(&config.pipelined_objectstore_bucket)
		.await
	{
		Ok(false) => {}
		Ok(true) => {
			info!(
				message = "Created storage bucket because it didn't exist",
				bucket = config.pipelined_objectstore_bucket
			);
		}
		Err(error) => {
			error!(
				message = "Error while creating storage bucket",
				bucket = config.pipelined_objectstore_bucket,
				?error
			);
		}
	}

	// Prep runner
	let mut runner: PipelineRunner<PipeData, CopperContext> =
		PipelineRunner::new(PipelineRunnerOptions {
			max_running_jobs: config.pipelined_max_running_jobs,
			job_log_size: config.pipelined_job_log_size,
			job_queue_size: config.pipelined_job_queue_size,
		});

	{
		// Base nodes
		use pipelined_basic::register;
		match register(runner.mut_dispatcher()) {
			Ok(()) => {}
			Err(error) => {
				error!(
					message = "Could not register nodes",
					module = "basic",
					?error
				);
				std::process::exit(1);
			}
		};
	}

	{
		// Storaged nodes
		use pipelined_storaged::register;
		match register(runner.mut_dispatcher()) {
			Ok(()) => {}
			Err(error) => {
				error!(
					message = "Could not register nodes",
					module = "storaged",
					?error
				);
				std::process::exit(1);
			}
		};
	}

	{
		// Audiofile nodes
		use pipelined_audiofile::nodes::register;
		match register(runner.mut_dispatcher()) {
			Ok(()) => {}
			Err(error) => {
				error!(
					message = "Could not register nodes",
					module = "audiofile",
					?error
				);
				std::process::exit(1);
			}
		};
	}

	trace!(message = "Initializing storaged client");
	let storaged_client = match ReqwestStoragedClient::new(
		config.pipelined_storaged_addr.clone(),
		&config.pipelined_storaged_secret,
	) {
		Ok(x) => Arc::new(x),
		Err(error) => {
			error!(message = "Could not initialize storaged client", ?error);
			std::process::exit(1);
		}
	};
	trace!(message = "Successfully initialized storaged client");

	let state = RouterState {
		runner: Arc::new(Mutex::new(runner)),
		storaged_client,
		objectstore_client: Arc::new(client),
		config,
	};

	let listener =
		match tokio::net::TcpListener::bind(state.config.pipelined_server_addr.to_string()).await {
			Ok(x) => x,
			Err(e) => {
				match e.kind() {
					std::io::ErrorKind::AddrInUse => {
						error!(
							message = "Cannot bind to port, already in use",
							port = state.config.pipelined_server_addr.as_str()
						);
					}
					_ => {
						error!(message = "Error while migrating main database", err = ?e);
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

	let app = api::router(state.clone());

	// Main loop(s)
	match tokio::try_join!(
		run_pipes(state),
		// Call .into on the error axum returns
		// so that the error types of all futures
		// in this join have the same type.
		//
		// The type of error is inferred from the first arg of this join.
		axum::serve(listener, app)
			.into_future()
			.map_err(|x| x.into())
	) {
		Ok(_) => {}
		Err(e) => {
			error!(message = "Main loop exited with error", err = e)
		}
	};
}

async fn run_pipes(state: RouterState) -> Result<(), Box<dyn Error>> {
	loop {
		let mut runner = state.runner.lock().await;
		runner.run().await?;

		// Sleep a little bit so we don't waste cpu cycles.
		tokio::time::sleep(std::time::Duration::from_millis(ASYNC_POLL_AWAIT_MS)).await;
	}
}
