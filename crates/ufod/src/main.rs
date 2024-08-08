use axum::{
	extract::{DefaultBodyLimit, State},
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
	pipeline::{AddJobParams, AddJobResult},
};
use ufo_database::{api::UFODatabase, database::Database};
use ufo_db_blobstore::fs::store::FsBlobstore;
use ufo_db_metastore::sqlite::db::SQLiteMetastore;
use ufo_db_pipestore::fs::FsPipestore;
use ufo_pipeline::{
	api::PipelineNodeStub,
	runner::runner::{PipelineRunConfig, PipelineRunner},
};
use ufo_pipeline_nodes::{
	data::{UFOData, UFODataStub},
	nodetype::UFONodeType,
	UFOContext,
};

mod pipeline;
mod status;

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
		config: Arc::new(config),
		runner: Arc::new(Mutex::new(runner)),
		database: Arc::new(database),
		context: Arc::new(ctx),
		uploader: Arc::new(Uploader::new("./tmp".into())),
	};

	let app = Router::new()
		.route("/", get(root))
		.route("/add_job", post(add_job))
		//
		.nest("/upload", Uploader::get_router(state.uploader.clone()))
		.nest("/pipelines", pipeline::router())
		.nest("/status", status::router())
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

async fn add_job(
	State(state): State<RouterState>,
	Json(payload): Json<AddJobParams>,
) -> impl IntoResponse {
	let mut runner = state.runner.lock().await;
	let db = state.database;

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
