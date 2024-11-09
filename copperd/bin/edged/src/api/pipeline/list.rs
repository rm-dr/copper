use crate::database::base::client::DatabaseClient;
use crate::database::base::errors::pipeline::ListPipelineError;
use crate::RouterState;
use axum::Json;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_itemdb::client::base::client::ItemdbClient;
use tracing::error;

/// List the logged in user's pipelines
#[utoipa::path(
	get,
	path = "/list",
	responses(
		(status = 200, description = "List of pipeline ids", body = Vec<PipelineInfo>),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn list_pipelines<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	match state.db_client.list_pipelines(user.id).await {
		Ok(pipelines) => return (StatusCode::OK, Json(pipelines)).into_response(),
		Err(ListPipelineError::DbError(error)) => {
			error!(
				message = "Database error while listing pipelines",
				user_id = ?user.id,
				?error,
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, Json("Internal server error")).into_response();
		}
	};
}
