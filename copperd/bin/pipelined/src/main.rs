use api::RouterState;
use config::PipelinedConfig;
use futures::TryFutureExt;
use pipelined_node_base::{data::CopperData, CopperContext};
use std::{error::Error, future::IntoFuture, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

use pipelined_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};

mod api;
mod config;

#[tokio::main]
async fn main() {
	// TODO: configure with env vars
	let config = Arc::new(PipelinedConfig::default());

	tracing_subscriber::fmt()
		.with_env_filter(config.to_env_filter())
		.without_time()
		.with_ansi(true)
		.init();

	// Prep runner
	let mut runner: PipelineRunner<CopperData, CopperContext> =
		PipelineRunner::new(PipelineRunConfig {
			node_threads: config.threads_per_job,
			max_active_jobs: config.parallel_jobs,
		});

	{
		// Base nodes
		use pipelined_node_base::nodes::register;
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

	let listener = match tokio::net::TcpListener::bind(state.config.server_addr.to_string()).await {
		Ok(x) => x,
		Err(e) => {
			match e.kind() {
				std::io::ErrorKind::AddrInUse => {
					error!(
						message = "Cannot bind to port, already in use",
						port = state.config.server_addr.as_str()
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
