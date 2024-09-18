use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	routing::{delete, get, patch, post},
	Router,
};
use copper_edged::PipelineInfo;
use copper_pipelined::{
	base::NodeParameterValue,
	json::{EdgeJson, InputPort, NodeJson, OutputPort, PipelineJson},
};
use utoipa::OpenApi;

mod add;
mod del;
mod get;
mod update;

use add::*;
use del::*;
use get::*;
use update::*;

#[allow(non_camel_case_types)]
#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(add_pipeline, update_pipeline, del_pipeline, get_pipeline),
	components(schemas(
		PipelineJson,
		NodeJson,
		EdgeJson,
		OutputPort,
		InputPort,
		NewPipelineRequest,
		UpdatePipelineRequest,
		NodeParameterValue,
		PipelineInfo
	))
)]
pub(super) struct PipelineApi;

pub(super) fn router<Client: DatabaseClient + 'static>() -> Router<RouterState<Client>> {
	Router::new()
		.route("/", post(add_pipeline))
		.route("/:pipeline_id", get(get_pipeline))
		.route("/:pipeline_id", delete(del_pipeline))
		.route("/:pipeline_id", patch(update_pipeline))
}
