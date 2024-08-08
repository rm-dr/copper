use crate::RouterState;
use axum::{extract::State, response::IntoResponse, Json};
use ufo_database::api::UFODatabase;

/// Get all pipelines
#[utoipa::path(
	get,
	path = "",
	responses(
		(status = 200, description = "Pipeline names", body = Vec<String>),
	),
)]
pub(super) async fn get_all_pipelines(State(state): State<RouterState>) -> impl IntoResponse {
	return Json(state.database.get_pipestore().all_pipelines().clone());
}
