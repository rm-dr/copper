use axum::{extract::DefaultBodyLimit, routing::get, Router};
use futures::executor::block_on;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use ufo_database::{api::UFODatabase, database::Database};
use ufo_db_blobstore::fs::store::FsBlobstore;
use ufo_db_metastore::sqlite::db::SQLiteMetastore;
use ufo_db_pipestore::fs::FsPipestore;
use ufo_pipeline::runner::runner::{PipelineRunConfig, PipelineRunner};
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};

mod pipeline;
mod status;

mod config;
mod upload;
use upload::uploader::Uploader;

#[derive(Clone)]
pub struct RouterState {
	config: Arc<config::UfodConfig>,
	runner: Arc<Mutex<PipelineRunner<UFONodeType>>>,
	database: Arc<Database<FsBlobstore, SQLiteMetastore, FsPipestore>>,
	context: Arc<UFOContext>,
	uploader: Arc<Uploader>,
}

// TODO: guaranteed unique pipeline job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)

// TODO: fix utoipa tags
#[derive(OpenApi)]
#[openapi(
	nest(
		(path = "/status", api = status::StatusApi),
		(path = "/pipelines", api = pipeline::PipelineApi),
		(path = "/upload", api = upload::UploadApi)
	),
	tags(
		(name = "ufod", description = "UFO backend daemon")
	),
)]
struct ApiDoc;

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
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		.route("/", get(root))
		//
		.nest("/upload", upload::router(state.uploader.clone()))
		.nest("/pipelines", pipeline::router())
		.nest("/status", status::router())
		//
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
