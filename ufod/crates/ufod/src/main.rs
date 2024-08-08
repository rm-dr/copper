use api::RouterState;
use futures::executor::block_on;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::Mutex;
use ufo_ds_impl::local::LocalDataset;

use ufo_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};

mod api;
mod config;

mod helpers;
use helpers::uploader::Uploader;

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
		//.event_format(log::LogFormatter::new(true))
		.init();

	//let mut f = File::open("./config.toml").unwrap();
	//let mut config_string = String::new();
	//f.read_to_string(&mut config_string).unwrap();
	//let config = toml::from_str(&config_string).unwrap();
	let config = Default::default();

	let database = Arc::new(LocalDataset::open(&PathBuf::from("./db")).unwrap());

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
		config: Arc::new(config),
		runner: Arc::new(Mutex::new(runner)),
		database: database.clone(),
		context: Arc::new(ctx),
		uploader: Arc::new(Uploader::new("./tmp".into())),
	};

	let listener = tokio::net::TcpListener::bind(state.config.server_addr.to_string())
		.await
		.unwrap();
	tracing::debug!("listening on {}", listener.local_addr().unwrap());

	let app = api::router(state.clone());

	thread::spawn(move || loop {
		let mut runner = block_on(state.runner.lock());
		runner.run().unwrap();
		block_on(state.uploader.check_jobs(&state.config, &runner));
		drop(runner);

		std::thread::sleep(std::time::Duration::from_millis(10));
	});

	axum::serve(listener, app).await.unwrap();
}
