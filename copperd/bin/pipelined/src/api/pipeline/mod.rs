use axum::{routing::post, Router};
use utoipa::OpenApi;

mod run;

use run::*;

use super::RouterState;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(run_pipeline),
	components(schemas(AddJobInput, AddJobRequest, AddJobResponse))
)]
pub(super) struct PipelineApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new().route("/run", post(run_pipeline))
}
