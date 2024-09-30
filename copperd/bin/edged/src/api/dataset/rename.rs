use crate::database::base::client::DatabaseClient;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_storaged::client::StoragedRequestError;
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct RenameDatasetRequest {
	pub new_name: String,
}

/// Rename a dataset
#[utoipa::path(
	patch,
	path = "/{dataset_id}",
	params(
		("dataset_id", description = "Dataset id"),
	),
	responses(
		(status = 200, description = "Dataset renamed successfully"),
		(status = 400, description = "Invalid request", body = String),
		(status = 401, description = "Unauthorized"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn rename_dataset<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(dataset_id): Path<i64>,
	Json(payload): Json<RenameDatasetRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	match state.storaged_client.get_dataset(dataset_id.into()).await {
		Ok(None) => return StatusCode::NOT_FOUND.into_response(),

		Ok(Some(x)) => {
			// We can only rename our own datasets
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}

		Err(StoragedRequestError::Other { error }) => {
			error!(message = "Error in storaged client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(StoragedRequestError::GenericHttp { code, message }) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}
	};

	let res = state
		.storaged_client
		.rename_dataset(dataset_id.into(), &payload.new_name)
		.await;

	return match res {
		Ok(()) => StatusCode::OK.into_response(),

		Err(StoragedRequestError::Other { error }) => {
			error!(message = "Error in storaged client", ?error);
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}

		Err(StoragedRequestError::GenericHttp { code, message }) => {
			if let Some(msg) = message {
				return (code, msg).into_response();
			} else {
				return code.into_response();
			}
		}
	};
}
