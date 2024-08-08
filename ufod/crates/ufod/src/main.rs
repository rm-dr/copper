use api::RouterState;
use config::UfodConfig;
use futures::executor::block_on;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::Mutex;
use tracing::info;
use ufo_ds_impl::local::LocalDataset;

use ufo_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};

mod api;
mod config;

mod helpers;
use helpers::{maindb::MainDB, uploader::Uploader};

// TODO: guaranteed unique pipeline job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(concat!(
			"ufo_pipeline=debug,sqlx=warn,tower_http=info,debug"
		))
		.without_time()
		.with_ansi(true)
		.init();

	//let mut f = File::open("./config.toml").unwrap();
	//let mut config_string = String::new();
	//f.read_to_string(&mut config_string).unwrap();
	//let config = toml::from_str(&config_string).unwrap();
	let config: UfodConfig = Default::default();

	// Open main database
	if !config.main_db.exists() {
		info!(
			message = "Creating main database because it doesn't exist",
			main_db_path = ?config.main_db
		);
		MainDB::create(&config.main_db).unwrap();
	}

	// TODO: arc config?
	let main_db = MainDB::open(config.clone()).unwrap();
	let uploader = Uploader::open(config.clone());

	//LocalDataset::create(&PathBuf::from("./data/db")).unwrap();
	let database = Arc::new(LocalDataset::open(&PathBuf::from("./data/db")).unwrap());

	let ctx = UFOContext {
		dataset: database.clone(),
		blob_fragment_size: 1_000_000,
	};

	// Prep runner
	let runner: PipelineRunner<UFONodeType> = PipelineRunner::new(
		PipelineRunConfig {
			node_threads: 2,
			max_active_jobs: 8,
		},
		ctx.clone(),
	);

	// TODO: clone fewer arcs
	let state = RouterState {
		main_db: Arc::new(main_db),
		config: Arc::new(config),
		runner: Arc::new(Mutex::new(runner)),
		database: database.clone(),
		context: Arc::new(ctx),
		uploader: Arc::new(uploader),
	};

	let listener = tokio::net::TcpListener::bind(state.config.server_addr.to_string())
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
