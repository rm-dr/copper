use api::RouterState;
use config::CopperConfig;
use futures::TryFutureExt;
use std::{error::Error, future::IntoFuture, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;
use ufo_node_base::{data::CopperData, CopperContext};

use ufo_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};

mod api;
mod config;
mod maindb;
mod uploader;

// TODO: guaranteed unique pipeline job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let config_path: PathBuf = "./data/config.toml".into();
	if !config_path.exists() {
		// We cannot log here, logger hasn't been initialized
		println!(
			"Generating default config at {}",
			config_path.to_str().unwrap()
		);
		CopperConfig::create_default_config(&config_path).unwrap();
	} else if !config_path.is_file() {
		// We cannot log here, logger hasn't been initialized
		println!(
			"Config path `{}` isn't a file, cannot start",
			config_path.to_str().unwrap()
		);
		std::process::exit(0);
	}

	let config = Arc::new(CopperConfig::load_from_file(&config_path).unwrap());

	// Logging is available after this point
	tracing_subscriber::fmt()
		.with_env_filter(config.logging.level.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	// Open main database
	if !config.paths.main_db.exists() {
		info!(
			message = "Creating main database because it doesn't exist",
			main_db_path = ?config.paths.main_db
		);
		maindb::MainDB::create(&config.paths.main_db).await.unwrap();
	}

	let main_db = maindb::MainDB::open(config.clone()).await.unwrap();
	let uploader = uploader::Uploader::open(config.clone());

	// Prep runner
	let mut runner: PipelineRunner<CopperData, CopperContext> =
		PipelineRunner::new(PipelineRunConfig {
			node_threads: config.pipeline.threads_per_job,
			max_active_jobs: config.pipeline.parallel_jobs,
		});

	{
		// Base nodes
		use ufo_node_base::nodes::register;
		register(runner.mut_dispatcher()).unwrap();
	}

	{
		// Audiofile nodes
		use ufo_audiofile::nodes::register;
		register(runner.mut_dispatcher()).unwrap();
	}

	// TODO: clone fewer arcs

	// Note how these are all async locks
	let state = RouterState {
		main_db: Arc::new(main_db),
		config,
		runner: Arc::new(Mutex::new(runner)),
		uploader: Arc::new(uploader),
	};

	let listener = tokio::net::TcpListener::bind(state.config.network.server_addr.to_string())
		.await
		.unwrap();
	info!("listening on {}", listener.local_addr().unwrap());

	let app = api::router(state.clone());

	// Main loop(s)
	tokio::try_join!(
		run_pipes(state),
		// Call .into on the error axum returns
		// so that the error types of all futures
		// in this join have the same type.
		//
		// The type of error is inferred from the first arg of this join.
		axum::serve(listener, app)
			.into_future()
			.map_err(|x| x.into())
	)?;

	return Ok(());
}

async fn run_pipes(state: RouterState) -> Result<(), Box<dyn Error>> {
	loop {
		let mut runner = state.runner.lock().await;
		runner.run();
		state.uploader.check_jobs(&runner).await;
		drop(runner);

		// Sleep a little bit so we don't waste cpu cycles.
		// If this is too long, we'll slow down pipeline runners,
		// but if it's too short we'll waste cycles checking pending threads.
		tokio::time::sleep(std::time::Duration::from_millis(10)).await;
	}
}
