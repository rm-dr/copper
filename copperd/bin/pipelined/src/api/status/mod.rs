use axum::{routing::get, Router};
use utoipa::OpenApi;

mod status;
use status::*;

use super::RouterState;

#[derive(OpenApi)]
#[openapi(tags(), paths(get_status), components(schemas(StatusResponse)))]
pub(super) struct StatusApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new().route("/", get(get_status))
}
