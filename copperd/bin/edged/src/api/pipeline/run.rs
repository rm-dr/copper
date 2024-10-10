use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_edged::ApiAttrData;
use copper_pipelined::client::PipelinedRequestError;
use copper_storaged::AttrData;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tracing::error;
use utoipa::ToSchema;

use crate::{
	database::base::{client::DatabaseClient, errors::pipeline::GetPipelineError},
	RouterState,
};

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct RunPipelineRequest {
	/// A unique id for this job
	#[schema(value_type = String)]
	pub job_id: SmartString<LazyCompact>,

	#[schema(value_type = BTreeMap<String, ApiAttrData>)]
	pub input: BTreeMap<SmartString<LazyCompact>, ApiAttrData>,
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/{pipeline_id}/run",
	params(
		("pipeline_id", description = "Pipeline id"),
	),
	responses(
		(status = 200, description = "Job queued successfully"),
		(status = 401, description = "Unauthorized"),
		(status = 409, description = "Job id already exists"),
		(status = 429, description = "Job queue is full"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn run_pipeline<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Path(pipeline_id): Path<i64>,
	Json(payload): Json<RunPipelineRequest>,
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
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	// Users can only get pipelines they own
	if pipe.owned_by != user.id {
		return StatusCode::UNAUTHORIZED.into_response();
	}

	let mut converted_input: BTreeMap<SmartString<LazyCompact>, AttrData> = BTreeMap::new();
	for (k, v) in payload.input {
		// If we can automatically convert, do so
		if let Ok(x) = AttrData::try_from(&v) {
			converted_input.insert(k, x);
			continue;
		}

		// Some types need manual conversion
		if let Some(x) = match &v {
			ApiAttrData::Blob { key } => Some(AttrData::Blob { key: key.clone() }),
			_ => None,
		} {
			converted_input.insert(k, x);
		}

		unreachable!("User-provided data {v:?} could not be converted automatically, but was not caught by the manual conversion `match`.")
	}

	let res = state
		.pipelined_client
		.run_pipeline(&pipe.data, &payload.job_id, &converted_input, user.id)
		.await;

	return match res {
		Ok(()) => StatusCode::OK.into_response(),

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
