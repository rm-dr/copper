use axum::{
	extract::{DefaultBodyLimit, Path, State},
	response::IntoResponse,
	routing::{get, post},
	Json, Router,
};
use config::UfodConfig;
use futures::executor::block_on;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use ufo_api::{
	data::{ApiData, ApiDataStub},
	pipeline::{AddJobParams, AddJobResult, NodeInfo, PipelineInfo},
	runner::{
		CompletedJobStatus, RunnerStatus, RunningJobStatus, RunningNodeState, RunningNodeStatus,
	},
};
use ufo_database::{api::UFODatabase, database::Database};
use ufo_db_blobstore::fs::store::FsBlobstore;
use ufo_db_metastore::sqlite::db::SQLiteMetastore;
use ufo_db_pipestore::fs::FsPipestore;
use ufo_pipeline::{
	api::{PipelineNodeState, PipelineNodeStub},
	labels::{PipelineLabel, PipelineNodeLabel},
	runner::runner::{PipelineRunConfig, PipelineRunner},
};
use ufo_pipeline_nodes::{
	data::{UFOData, UFODataStub},
	nodetype::UFONodeType,
	UFOContext,
};

mod config;
mod upload;
use upload::Uploader;

#[derive(Clone)]
pub struct RouterState {
	config: Arc<UfodConfig>,
	runner: Arc<Mutex<PipelineRunner<UFONodeType>>>,
	database: Arc<Database<FsBlobstore, SQLiteMetastore, FsPipestore>>,
	context: Arc<UFOContext>,
	uploader: Arc<Uploader>,
}

// TODO: openapi
// TODO: guaranteed unique job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)
// client checks server vserion

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(concat!(
			"ufo_pipeline=error,sqlx=warn,tower_http=info,debug"
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

	let database = Database::open(&PathBuf::from("./db")).unwrap();

	let ctx = UFOContext {
		metastore: database.get_metastore(),
		blobstore: database.get_blobstore(),
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
		config,
		runner: Arc::new(Mutex::new(runner)),
		database: Arc::new(database),
		context: Arc::new(ctx),
		uploader: Arc::new(Uploader::new("./tmp".into())),
	};

	let app = Router::new()
		.route("/", get(root))
		// Status endpoints
		.route("/status", get(get_status))
		.route("/status/completed", get(get_completed))
		// Pipeline endpoints
		.route("/pipelines", get(get_all_pipelines))
		.route("/pipelines/:pipeline_name", get(get_pipeline))
		.route(
			"/pipelines/:pipeline_name/:node_name",
			get(get_pipeline_node),
		)
		// Job endpoints
		.route("/add_job", post(add_job))
		.nest("/upload", Uploader::get_router(state.uploader.clone()))
		// Finish
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.request_body_limit))
		.with_state(state.clone());

	let listener = tokio::net::TcpListener::bind(state.config.server_addr.to_string())
		.await
		.unwrap();
	tracing::debug!("listening on {}", listener.local_addr().unwrap());

	thread::spawn(move || loop {
		let mut runner = block_on(state.runner.lock());
		runner.run().unwrap();
		block_on(state.uploader.check_jobs(&state.config, &runner));
		drop(runner);

		std::thread::sleep(std::time::Duration::from_millis(10));
	});

	axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
	"Hello, World!"
}

async fn get_status(State(state): State<RouterState>) -> impl IntoResponse {
	let runner = state.runner.lock().await;

	let running_jobs: Vec<RunningJobStatus> = runner
		.iter_active_jobs()
		.map(|(job_id, job)| {
			let p = job.get_pipeline();
			RunningJobStatus {
				job_id: *job_id,
				pipeline: p.get_name().clone(),
				input_exemplar: format!("{:?}", job.get_input().first().unwrap()),
				node_status: p
					.iter_node_labels()
					.map(|l| RunningNodeStatus {
						name: l.clone(),
						state: match job.get_node_status(l).unwrap() {
							(true, _) => RunningNodeState::Running,
							(false, PipelineNodeState::Done) => RunningNodeState::Done,
							(false, PipelineNodeState::Pending(m)) => {
								RunningNodeState::Pending { message: m.into() }
							}
						},
					})
					.collect(),
			}
		})
		.collect();

	return Json(RunnerStatus {
		queued_jobs: runner.get_queued_jobs().len(),
		finished_jobs: runner.get_completed_jobs().len(),
		running_jobs,
	});
}

/// Get all pipeline names
async fn get_all_pipelines(State(state): State<RouterState>) -> impl IntoResponse {
	return Json(state.database.get_pipestore().all_pipelines().clone());
}

/// Get details about one pipeline
async fn get_pipeline(
	Path(pipeline_name): Path<PipelineLabel>,
	State(state): State<RouterState>,
) -> impl IntoResponse {
	let pipe = if let Some(pipe) = state
		.database
		.get_pipestore()
		.load_pipeline(&pipeline_name, state.context)
	{
		pipe
	} else {
		return Json(None);
	};

	let nodes = pipe.iter_node_labels().cloned().collect::<Vec<_>>();

	return Json(Some(PipelineInfo {
		name: pipeline_name,
		nodes,
		input_node: pipe.input_node_label().clone(),
		output_node: pipe.output_node_label().clone(),
	}));
}

/// Get details about a node in one pipeline
async fn get_pipeline_node(
	Path((pipeline_name, node_name)): Path<(PipelineLabel, PipelineNodeLabel)>,
	State(state): State<RouterState>,
) -> impl IntoResponse {
	let pipe = if let Some(pipe) = state
		.database
		.get_pipestore()
		.load_pipeline(&pipeline_name, state.context.clone())
	{
		pipe
	} else {
		return Json(None);
	};

	let node = if let Some(node) = pipe.get_node(&node_name) {
		node
	} else {
		return Json(None);
	};

	let inputs = (0..node.n_inputs(&state.context))
		.map(|i| {
			UFODataStub::iter_all()
				.filter(|stub| node.input_compatible_with(&state.context, i, **stub))
				.map(|x| match x {
					UFODataStub::Text => ApiDataStub::Text,
					UFODataStub::Path => ApiDataStub::Blob,
					UFODataStub::Binary => todo!(),
					UFODataStub::Blob => todo!(),
					UFODataStub::Integer => ApiDataStub::Integer,
					UFODataStub::PositiveInteger => ApiDataStub::PositiveInteger,
					UFODataStub::Boolean => ApiDataStub::Boolean,
					UFODataStub::Float => ApiDataStub::Float,
					UFODataStub::Hash { .. } => todo!(),
					UFODataStub::Reference { .. } => todo!(),
				})
				.collect()
		})
		.collect::<Vec<_>>();

	return Json(Some(NodeInfo {
		name: node_name,
		inputs,
	}));
}

async fn get_completed(State(state): State<RouterState>) -> impl IntoResponse {
	let runner = state.runner.lock().await;

	let completed_jobs: Vec<CompletedJobStatus> = runner
		.get_completed_jobs()
		.iter()
		.map(|c| CompletedJobStatus {
			job_id: c.job_id,
			pipeline: c.pipeline.clone(),
			error: c.error.as_ref().map(|x| x.to_string()),
			input_exemplar: format!("{:?}", c.input.first().unwrap()),
		})
		.collect();

	return Json(completed_jobs);
}

async fn add_job(
	State(state): State<RouterState>,
	Json(payload): Json<AddJobParams>,
) -> impl IntoResponse {
	let mut runner = state.runner.lock().await;
	let db = state.database;

	debug!(message="Got request to add job", payload=?payload);

	let pipeline = if let Some(pipeline) = db
		.get_pipestore()
		.load_pipeline(&payload.pipeline, state.context)
	{
		// TODO: cache pipelines
		pipeline
	} else {
		return Json(AddJobResult::BadPipeline {
			pipeline: payload.pipeline,
		});
	};

	let ctx = runner.get_context();
	let in_node = pipeline.input_node_label();
	let in_node = pipeline.get_node(in_node).unwrap();

	// Check number of arguments
	let expected_inputs = in_node.n_inputs(ctx);
	if expected_inputs != payload.input.len() {
		return Json(AddJobResult::InvalidNumberOfArguments {
			got: payload.input.len(),
			expected: expected_inputs,
		});
	}

	// Check type of each argument
	for (i, data) in payload.input.iter().enumerate() {
		let t = match data {
			ApiData::None(t) => match t {
				ApiDataStub::Text => UFODataStub::Text,
				ApiDataStub::Blob => UFODataStub::Path,
				ApiDataStub::Integer => UFODataStub::Integer,
				ApiDataStub::PositiveInteger => UFODataStub::PositiveInteger,
				ApiDataStub::Boolean => UFODataStub::Boolean,
				ApiDataStub::Float => UFODataStub::Float,
			},
			ApiData::Text(_) => UFODataStub::Text,
			ApiData::Blob { .. } => UFODataStub::Path,
			ApiData::Integer(_) => UFODataStub::Integer,
			ApiData::PositiveInteger(_) => UFODataStub::PositiveInteger,
			ApiData::Boolean(_) => UFODataStub::Boolean,
			ApiData::Float(_) => UFODataStub::Float,
		};

		if !in_node.input_compatible_with(ctx, 0, t) {
			return Json(AddJobResult::InvalidInputType { bad_input_idx: i });
		}
	}

	let mut inputs = Vec::new();
	for i in payload.input {
		let x = match i {
			ApiData::None(t) => UFOData::None(match t {
				ApiDataStub::Text => UFODataStub::Text,
				ApiDataStub::Blob => UFODataStub::Path,
				ApiDataStub::Integer => UFODataStub::Integer,
				ApiDataStub::PositiveInteger => UFODataStub::PositiveInteger,
				ApiDataStub::Boolean => UFODataStub::Boolean,
				ApiDataStub::Float => UFODataStub::Float,
			}),
			ApiData::Text(t) => UFOData::Text(Arc::new(t)),
			ApiData::Blob { file_name } => {
				let j = payload.bound_upload_job.as_ref();

				if j.is_none() {
					panic!();
				}
				let j = j.unwrap();

				if !state
					.uploader
					.has_file_been_finished(j, &file_name)
					.await
					.unwrap()
				{
					panic!("unfinished file!")
				}

				let p = state.uploader.get_job_file_path(j, &file_name).await;

				if let Some(p) = p {
					UFOData::Path(p)
				} else {
					panic!("bad job")
				}
			}
			ApiData::Integer(i) => UFOData::Integer(i),
			ApiData::PositiveInteger(i) => UFOData::PositiveInteger(i),
			ApiData::Boolean(b) => UFOData::Boolean(b),
			ApiData::Float(f) => UFOData::Float(f),
		};

		inputs.push(x);
	}

	let new_id = runner.add_job(Arc::new(pipeline), inputs);

	if let Some(j) = payload.bound_upload_job {
		state
			.uploader
			.bind_job_to_pipeline(&j, new_id)
			.await
			.unwrap();
	}

	return Json(AddJobResult::Ok);
}
