use crate::database::base::{client::DatabaseClient, errors::pipeline::GetPipelineError};
use crate::RouterState;
use axum::Json;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::client::ItemdbClient;
use tracing::error;

/// Get pipeline by id
#[utoipa::path(
	get,
	path = "/{pipeline_id}",
	params(
		("pipeline_id", description = "Pipeline id"),
	),
	responses(
		(status = 200, description = "Pipeline found", body = PipelineInfo),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn get_pipeline<Client: DatabaseClient, Itemdb: ItemdbClient>(
	// OriginalUri(uri): OriginalUri,
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(pipeline_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let pipe = match state.db_client.get_pipeline(pipeline_id.into()).await {
		Ok(Some(pipe)) => pipe,
		Ok(None) => return StatusCode::NOT_FOUND.into_response(),
		Err(GetPipelineError::DbError(error)) => {
			error!(
				message = "Database error while getting pipeline",
				?pipeline_id,
				?error,
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	// Users can only get pipelines they own
	if pipe.owned_by != user.id {
		return (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response();
	}

	return (StatusCode::OK, Json(pipe)).into_response();
}
