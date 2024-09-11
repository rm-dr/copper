use axum::{extract::DefaultBodyLimit, Router};
use copper_pipelined::{
	base::NodeParameterValue,
	data::{PipeData, PipeDataStub},
	CopperContext,
};
use copper_storaged::{client::StoragedClient, AttrData, AttrDataStub};
use copper_util::HashType;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use utoipa::{
	openapi::security::{Http, HttpAuthScheme, SecurityScheme},
	Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod pipeline;

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
	pub storaged_client: Arc<dyn StoragedClient>,
	pub objectstore_client: Arc<aws_sdk_s3::Client>,
}

struct BearerSecurityAddon;
impl Modify for BearerSecurityAddon {
	fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
		if let Some(components) = openapi.components.as_mut() {
			components.add_security_scheme(
				"bearer",
				SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
			)
		}
	}
}

#[derive(OpenApi)]
#[openapi(
	modifiers(&BearerSecurityAddon),
	nest(
		(path = "/pipeline", api = pipeline::PipelineApi),
	),
	tags(
		(name = "pipelined", description = "Copper pipeline runner")
	),
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
		.nest("/pipeline", pipeline::router())
		//
		.layer(TraceLayer::new_for_http())
		.layer(DefaultBodyLimit::max(
			state.config.pipelined_request_body_limit,
		))
		.with_state(state)
}
