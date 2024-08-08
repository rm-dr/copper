use axum::{extract::DefaultBodyLimit, routing::get, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use ufo_ds_core::{
	api::meta::AttributeOptions,
	data::{HashType, MetastoreDataStub},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::nodetype::UFONodeType;

mod attr;
mod class;
mod dataset;
mod pipeline;
mod status;
mod upload;

use crate::{
	config::UfodConfig,
	helpers::{maindb::MainDB, uploader::Uploader},
};

#[derive(Clone)]
pub struct RouterState {
	pub config: Arc<UfodConfig>,
	pub runner: Arc<Mutex<PipelineRunner<UFONodeType>>>,
	pub main_db: Arc<MainDB>,
	pub uploader: Arc<Uploader>,
}

// TODO: guaranteed unique pipeline job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)

// TODO: fix utoipa tags
#[derive(OpenApi)]
#[openapi(
	nest(
		(path = "/status", api = status::StatusApi),
		(path = "/upload", api = upload::UploadApi),
		(path = "/dataset", api = dataset::DatasetApi),
		(path = "/pipeline", api = pipeline::PipelineApi),
		(path = "/class", api = class::ClassApi),
		(path = "/attr", api = attr::AttrApi)
	),
	tags(
		(name = "ufod", description = "UFO backend daemon")
	),
	components(schemas(
		MetastoreDataStub,
		HashType,
		AttributeOptions
	))
)]
struct ApiDoc;

pub(super) fn router(state: RouterState) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		.route("/", get(root))
		//
		.nest("/upload", upload::router(state.uploader.clone()))
		.nest("/status", status::router())
		.nest("/dataset", dataset::router())
		.nest("/pipeline", pipeline::router())
		.nest("/class", class::router())
		.nest("/attr", attr::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.request_body_limit))
		.with_state(state)
}

async fn root() -> &'static str {
	"Hello, World!"
}
