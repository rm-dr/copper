use api::RouterState;
use config::PipelinedConfig;
use copper_pipelined::{data::PipeData, CopperContext};
use copper_util::load_env;
use futures::TryFutureExt;
use pipeline::runner::{PipelineRunner, PipelineRunnerOptions};
use std::{error::Error, future::IntoFuture, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

mod api;
mod config;
mod pipeline;

#[tokio::main]
async fn main() {
	let config = Arc::new(load_env::<PipelinedConfig>());

	tracing_subscriber::fmt()
		.with_env_filter(config.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	debug!(message = "Loaded config from environment", ?config);

	// Prep runner
	let mut runner: PipelineRunner<PipeData, CopperContext> =
		PipelineRunner::new(PipelineRunnerOptions {
			node_threads: config.pipelined_threads_per_job,
			max_active_jobs: config.pipelined_parallel_jobs,
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
		config,
		runner: Arc::new(Mutex::new(runner)),
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
		runner.run();
		drop(runner);

		// Sleep a little bit so we don't waste cpu cycles.
		// If this is too long, we'll slow down pipeline runners,
		// but if it's too short we'll waste cycles checking pending threads.
		tokio::time::sleep(std::time::Duration::from_millis(10)).await;
	}
}
