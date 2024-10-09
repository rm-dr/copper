use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::extract::Query;
use axum::Json;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_pipelined::client::PipelinedRequestError;
use serde::Deserialize;
use tracing::error;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub(super) struct PaginateParams {
	skip: usize,
	count: usize,
}

/// List the logged in user's jobs
#[utoipa::path(
	get,
	path = "/list",
	params(PaginateParams),
	responses(
		(status = 200, description = "This user's jobs, ordered by age", body = JobInfoList),
		(status = 401, description = "Unauthorized"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn list_jobs<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Query(paginate): Query<PaginateParams>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	return match state
		.pipelined_client
		.list_user_jobs(user.id, paginate.skip, paginate.count)
		.await
	{
		Ok(x) => (StatusCode::OK, Json(x)).into_response(),

		Err(PipelinedRequestError::Other { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(PipelinedRequestError::GenericHttp { code, message }) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}
	};
}
