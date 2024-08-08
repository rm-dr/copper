use axum::{extract::DefaultBodyLimit, routing::get, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use ufo_ds_core::api::Dataset;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use ufo_pipeline::runner::runner::PipelineRunner;
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};

mod dataset;
mod pipeline;
mod status;
pub mod upload;

use crate::{config::UfodConfig, helpers::uploader::Uploader};

#[derive(Clone)]
pub struct RouterState {
	pub config: Arc<UfodConfig>,
	pub runner: Arc<Mutex<PipelineRunner<UFONodeType>>>,
	pub database: Arc<dyn Dataset<UFONodeType>>,
	pub context: Arc<UFOContext>,
	pub uploader: Arc<Uploader>,
}

// TODO: guaranteed unique pipeline job id (?)
// delete after timeout (what if uploading takes a while? Multiple big files?)

// TODO: fix utoipa tags
#[derive(OpenApi)]
#[openapi(
	nest(
		(path = "/status", api = status::StatusApi),
		(path = "/pipelines", api = pipeline::PipelineApi),
		(path = "/upload", api = upload::UploadApi),
		(path = "/dataset", api = dataset::DatasetApi)
	),
	tags(
		(name = "ufod", description = "UFO backend daemon")
	),
)]
struct ApiDoc;

pub(super) fn router(state: RouterState) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		.route("/", get(root))
		//
		.nest("/upload", upload::router(state.uploader.clone()))
		.nest("/pipelines", pipeline::router())
		.nest("/status", status::router())
		.nest("/dataset", dataset::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(state.config.request_body_limit))
		.with_state(state)
}

async fn root() -> &'static str {
	"Hello, World!"
}
