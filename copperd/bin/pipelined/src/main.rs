use api::RouterState;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use config::{PipelinedConfig, ASYNC_POLL_AWAIT_MS};
use copper_pipelined::{data::PipeData, helpers::S3Client, CopperContext};
use copper_storaged::client::ReqwestStoragedClient;
use copper_util::load_env;
use futures::TryFutureExt;
use pipeline::runner::{PipelineRunner, PipelineRunnerOptions};
use std::{error::Error, future::IntoFuture, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

mod api;
mod config;
mod pipeline;

// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() {
	let config = Arc::new(load_env::<PipelinedConfig>());

	tracing_subscriber::fmt()
		.with_env_filter(config.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	debug!(message = "Loaded config from environment", ?config);

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

	let client = Arc::new(aws_sdk_s3::Client::from_conf(s3_config));

	// Prep runner
	let mut runner: PipelineRunner<PipeData, CopperContext> =
		PipelineRunner::new(PipelineRunnerOptions {
			max_running_jobs: config.pipelined_max_running_jobs,
		});

	{
		// Base nodes
		use pipelined_basic::register;
		register(runner.mut_dispatcher()).unwrap();
	}

	{
		// Storaged nodes
		use pipelined_storaged::register;
		register(runner.mut_dispatcher()).unwrap();
	}

	{
		// Audiofile nodes
		use pipelined_audiofile::nodes::register;
		register(runner.mut_dispatcher()).unwrap();
	}

	// Note how these are all async locks
	let state = RouterState {
		runner: Arc::new(Mutex::new(runner)),

		storaged_client: Arc::new(
			ReqwestStoragedClient::new(
				config.pipelined_storaged_addr.clone(),
				&config.pipelined_storaged_secret,
			)
			// TODO: handle error
			.unwrap(),
		),

		objectstore_client: Arc::new(
			S3Client::new(client.clone(), &config.pipelined_objectstore_bucket).await,
		),

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
	info!("listening on http://{}", listener.local_addr().unwrap());

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
