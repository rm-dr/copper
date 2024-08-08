use crate::RouterState;
use axum::{routing::get, Router};
use utoipa::OpenApi;

mod apidata;
mod node;
mod pipeline;
mod pipelines;

use node::*;
use pipeline::*;
use pipelines::*;

#[derive(OpenApi)]
#[openapi(
	paths(get_all_pipelines, get_pipeline, get_pipeline_node),
	components(schemas(PipelineInfo, NodeInfo, apidata::ApiData, apidata::ApiDataStub))
)]
pub(super) struct PipelineApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/", get(get_all_pipelines))
		.route("/:pipeline_name", get(get_pipeline))
		.route("/:pipeline_name/:node_name", get(get_pipeline_node))
}
