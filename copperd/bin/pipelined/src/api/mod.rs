use axum::{extract::DefaultBodyLimit, Router};
use copper_pipelined::{
	base::NodeParameterValue,
	data::{PipeData, PipeDataStub},
	CopperContext,
};
use copper_storaged::{AttrData, AttrDataStub};
use copper_util::HashType;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod pipeline;
mod status;

use crate::{
	config::PipelinedConfig,
	pipeline::{
		json::{EdgeJson, EdgeType, InputPort, NodeJson, OutputPort, PipelineJson},
		runner::PipelineRunner,
	},
};

#[derive(Clone)]
pub struct RouterState {
	pub config: Arc<PipelinedConfig>,
	pub runner: Arc<Mutex<PipelineRunner<PipeData, CopperContext>>>,
}

#[derive(OpenApi)]
#[openapi(
	//modifiers(&BearerSecurityAddon),
	nest(
		(path = "/status", api = status::StatusApi),
		(path = "/pipeline", api = pipeline::PipelineApi),
	),
	tags(
		(name = "Copper", description = "Copper backend daemon")
	),
	// All schema structs defined outside `crate::api` go here
	components(schemas(
		PipeDataStub,
		PipeData,
		PipelineJson<PipeData>,
		NodeJson<PipeData>,
		EdgeJson,
		OutputPort,
		InputPort,
		EdgeType,
		NodeParameterValue<PipeData>,
		AttrData,
		AttrDataStub,
		HashType
	))
)]
struct ApiDoc;

pub(super) fn router(state: RouterState) -> Router {
	Router::new()
		.merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
		//
		.nest("/status", status::router())
		.nest("/pipeline", pipeline::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(
			state.config.pipelined_request_body_limit,
		))
		.with_state(state)
}
