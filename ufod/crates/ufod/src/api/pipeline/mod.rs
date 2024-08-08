use axum::{
	routing::{get, post},
	Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};

mod get;
mod list;
mod run;

use get::*;
use list::*;
use run::*;

use super::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug, IntoParams)]
pub(in crate::api) struct PipelineSelect {
	pub dataset: String,
	pub pipeline: String,
}

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(get_pipeline, list_pipelines, run_pipeline),
	components(schemas(
		PipelineSelect,
		AddJobInput,
		AddJobParams,
		PipelineListRequest,
		PipelineInfoShort,
		PipelineInfoInput,
		PipelineInfo
	))
)]
pub(super) struct PipelineApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		.route("/get", get(get_pipeline))
		.route("/list", get(list_pipelines))
		.route("/run", post(run_pipeline))
}
