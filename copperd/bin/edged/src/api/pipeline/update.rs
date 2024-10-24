use crate::database::base::{
	client::DatabaseClient,
	errors::pipeline::{GetPipelineError, UpdatePipelineError},
};
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_piper::json::PipelineJson;
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct UpdatePipelineRequest {
	pub new_name: Option<String>,
	pub new_data: Option<PipelineJson>,
}

/// Update a pipeline
#[utoipa::path(
	patch,
	path = "/{pipeline_id}",
	params(
		("pipeline_id", description = "Pipeline id"),
	),
	responses(
		(status = 200, description = "Pipeline updated successfully", body = PipelineInfo),
		(status = 400, description = "Invalid request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn update_pipeline<Client: DatabaseClient, Itemdb: ItemdbClient>(
	// OriginalUri(uri): OriginalUri,
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(pipeline_id): Path<i64>,
	Json(payload): Json<UpdatePipelineRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let mut pipe = match state.db_client.get_pipeline(pipeline_id.into()).await {
		Ok(Some(pipe)) => pipe,
		Ok(None) => return StatusCode::NOT_FOUND.into_response(),
		Err(GetPipelineError::DbError(error)) => {
			error!(
				message = "Database error while getting pipeline",
				?pipeline_id,
				?error,
			);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	// Users can only update pipelines they own
	if pipe.owned_by != user.id {
		return StatusCode::UNAUTHORIZED.into_response();
	}

	// Update pipeline info
	if let Some(name) = payload.new_name {
		pipe.name = name.into()
	}

	if let Some(data) = payload.new_data {
		pipe.data = data
	}

	// Save pipeline info
	let res = state.db_client.update_pipeline(&pipe).await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(UpdatePipelineError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a pipeline with this name already exists"),
		)
			.into_response(),

		Err(UpdatePipelineError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(UpdatePipelineError::DbError(e)) => {
			error!(
				message = "Database error while renaming pipeline",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
