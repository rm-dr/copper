use crate::database::base::{client::DatabaseClient, errors::pipeline::AddPipelineError};
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::client::ItemdbClient;
use copper_pipelined::json::PipelineJson;
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct NewPipelineRequest {
	name: String,
	pipeline: PipelineJson,
}

/// Create a new pipeline
#[utoipa::path(
	post,
	path = "",
	responses(
		(status = 200, description = "Pipeline created successfully", body = PipelineInfo),
		(status = 400, description = "Bad request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn add_pipeline<Client: DatabaseClient, Itemdb: ItemdbClient>(
	// OriginalUri(uri): OriginalUri,
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Json(payload): Json<NewPipelineRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let res = state
		.db_client
		.add_pipeline(user.id, &payload.name, &payload.pipeline)
		.await;

	return match res {
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(AddPipelineError::NameError(e)) => {
			(StatusCode::BAD_REQUEST, Json(format!("{}", e))).into_response()
		}

		Err(AddPipelineError::UniqueViolation) => (
			StatusCode::BAD_REQUEST,
			Json("a pipeline with this name already exists"),
		)
			.into_response(),

		Err(AddPipelineError::DbError(e)) => {
			error!(
				message = "Database error while making new pipeline",
				error = ?e
			);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	};
}
