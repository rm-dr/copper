use crate::RouterState;
use axum::{
	routing::{get, post},
	Router,
};
use utoipa::OpenApi;

mod node;
mod pipeline;
mod pipelines;
mod run;

use node::*;
use pipeline::*;
use pipelines::*;
use run::*;

#[derive(OpenApi)]
#[openapi(
	paths(get_all_pipelines, get_pipeline, get_pipeline_node, run_pipeline),
	components(schemas(
		PipelineInfoShort,
		PipelineInfoInput,
		PipelineInfo,
		NodeInfo,
		AddJobParams,
		AddJobResult,
		AddJobInput,
	))
)]
pub(super) struct PipelineApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/", get(get_all_pipelines))
		.route("/:pipeline_name", get(get_pipeline))
		.route("/:pipeline_name/run", post(run_pipeline))
		.route("/:pipeline_name/:node_id", get(get_pipeline_node))
}
