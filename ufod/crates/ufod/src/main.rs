use api::RouterState;
use config::UfodConfig;
use futures::executor::block_on;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::Mutex;
use tracing::info;

use ufo_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};
use ufo_pipeline_nodes::nodetype::UFONodeType;

mod api;
mod config;

mod helpers;
use helpers::{maindb::MainDB, uploader::Uploader};

// TODO: guaranteed unique pipeline job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)

#[tokio::main]
async fn main() {
	let config_path: PathBuf = "./data/config.toml".into();
	if !config_path.exists() {
		println!(
			"Generating default config at {}",
			config_path.to_str().unwrap()
		);
		UfodConfig::create_default_config(&config_path).unwrap();
	} else if !config_path.is_file() {
		println!(
			"Config path `{}` isn't a file, cannot start",
			config_path.to_str().unwrap()
		);
		std::process::exit(0);
	}

	let config = Arc::new(UfodConfig::load_from_file(&config_path).unwrap());

	// We cannot log before this point
	tracing_subscriber::fmt()
		.with_env_filter(config.logging.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	// Open main database
	if !config.paths.main_db.exists() {
		info!(
			message = "Creating main database because it doesn't exist",
			main_db_path = ?config.paths.main_db
		);
		MainDB::create(&config.paths.main_db).unwrap();
	}

	// TODO: arc config?
	let main_db = MainDB::open(config.clone()).unwrap();
	let uploader = Uploader::open(config.clone());

	// Prep runner
	let runner: PipelineRunner<UFONodeType> = PipelineRunner::new(PipelineRunConfig {
		node_threads: 2,
		max_active_jobs: 8,
	});

	// TODO: clone fewer arcs
	let state = RouterState {
		main_db: Arc::new(main_db),
		config,
		runner: Arc::new(Mutex::new(runner)),
		uploader: Arc::new(uploader),
	};

	let listener = tokio::net::TcpListener::bind(state.config.network.server_addr.to_string())
		.await
		.unwrap();
	tracing::debug!("listening on {}", listener.local_addr().unwrap());

	let app = api::router(state.clone());

	thread::spawn(move || loop {
		let mut runner = block_on(state.runner.lock());
		runner.run().unwrap();
		block_on(state.uploader.check_jobs(&runner));
		drop(runner);

		std::thread::sleep(std::time::Duration::from_millis(10));
	});

	axum::serve(listener, app).await.unwrap();
}
