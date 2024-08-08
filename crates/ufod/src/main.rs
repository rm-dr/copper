use std::{
	path::PathBuf,
	sync::{Arc, Mutex},
	thread,
};

use axum::{
	extract::State,
	http::StatusCode,
	response::IntoResponse,
	routing::{get, post},
	Json, Router,
};
use ufo_database::{api::UFODatabase, database::Database};
use ufo_pipeline::{
	api::PipelineNodeState,
	labels::PipelineLabel,
	runner::runner::{PipelineRunConfig, PipelineRunner},
};
use ufo_pipeline_nodes::{data::UFOData, nodetype::UFONodeType, UFOContext};
use ufod::{
	AddJobParams, CompletedJobStatus, RunnerStatus, RunningJobStatus, RunningNodeState,
	RunningNodeStatus,
};

#[derive(Clone)]
struct RouterState {
	runner: Arc<Mutex<PipelineRunner<UFONodeType>>>,
}

// TODO: openapi
// TODO: guaranteed unique job id (?)
// TODO: api response json tagging

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		//.with_env_filter("ufo_pipeline=debug")
		.with_env_filter("ufo_pipeline=error")
		.without_time()
		.with_ansi(true)
		//.with_max_level(Level::DEBUG)
		//.event_format(log::LogFormatter::new(true))
		.init();

	let database = Database::open(&PathBuf::from("./db")).unwrap();

	let all = database
		.get_pipestore()
		.all_pipelines()
		.iter()
		.map(|x| {
			(
				x.clone(),
				database.get_pipestore().load_pipeline(x.clone().into()),
			)
		})
		.collect::<Vec<_>>();

	let ctx = UFOContext {
		metastore: database.get_metastore(),
		blobstore: database.get_blobstore(),
		blob_fragment_size: 1_000_000,
	};

	// Prep runner
	let mut runner: PipelineRunner<UFONodeType> = PipelineRunner::new(
		PipelineRunConfig {
			node_threads: 2,
			max_active_jobs: 8,
		},
		ctx.clone(),
	);

	for (l, t) in all {
		runner.add_pipeline(l, t).unwrap();
	}

	let state = RouterState {
		runner: Arc::new(Mutex::new(runner)),
	};

	let app = Router::new()
		.route("/", get(root))
		.route("/status", get(get_status))
		.route("/status/completed", get(get_completed))
		.route("/add_job", post(add_job))
		.with_state(state.clone());

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
		.await
		.unwrap();
	tracing::debug!("listening on {}", listener.local_addr().unwrap());

	thread::spawn(move || loop {
		let mut runner = state.runner.lock().unwrap();
		runner.run().unwrap();
		drop(runner);
		std::thread::sleep(std::time::Duration::from_millis(10));
	});

	axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
	"Hello, World!"
}

async fn get_status(State(state): State<RouterState>) -> impl IntoResponse {
	let runner = state.runner.lock().unwrap();

	let running_jobs: Vec<RunningJobStatus> = runner
		.iter_active_jobs()
		.map(|(job_id, job)| {
			let p = job.get_pipeline();
			RunningJobStatus {
				job_id: *job_id,
				pipeline: p.get_name().to_string(),
				input_exemplar: format!("{:?}", job.get_input().first().unwrap()),
				node_status: p
					.iter_node_labels()
					.map(|l| RunningNodeStatus {
						name: l.to_string(),
						state: match job.get_node_status(l).unwrap() {
							(true, _) => RunningNodeState::Running,
							(false, PipelineNodeState::Done) => RunningNodeState::Done,
							(false, PipelineNodeState::Pending(m)) => {
								RunningNodeState::Pending(m.into())
							}
						},
					})
					.collect(),
			}
		})
		.collect();

	(
		StatusCode::OK,
		Json(RunnerStatus {
			queued_jobs: runner.get_queued_jobs().len(),
			finished_jobs: runner.get_completed_jobs().len(),
			running_jobs,
		}),
	)
}

async fn get_completed(State(state): State<RouterState>) -> impl IntoResponse {
	let runner = state.runner.lock().unwrap();

	let completed_jobs: Vec<CompletedJobStatus> = runner
		.get_completed_jobs()
		.iter()
		.map(|c| CompletedJobStatus {
			job_id: c.job_id,
			pipeline: c.pipeline.to_string(),
			error: c.error.as_ref().map(|x| x.to_string()),
			input_exemplar: format!("{:?}", c.input.first().unwrap()),
		})
		.collect();

	(StatusCode::OK, Json(completed_jobs))
}

async fn add_job(
	State(state): State<RouterState>,
	Json(payload): Json<AddJobParams>,
) -> impl IntoResponse {
	let mut runner = state.runner.lock().unwrap();

	let pipeline: PipelineLabel = payload.pipeline.into();
	if runner.get_pipeline(&pipeline).is_none() {
		return StatusCode::BAD_REQUEST;
	}

	runner.add_job(
		&"audiofile".into(),
		vec![UFOData::Path(Arc::new(payload.input))],
	);

	StatusCode::CREATED
}
