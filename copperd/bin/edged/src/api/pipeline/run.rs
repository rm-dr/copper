use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::{client::base::client::ItemdbClient, AttrData};
use copper_jobqueue::base::errors::AddJobError;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tracing::error;
use utoipa::ToSchema;

use crate::{
	apidata::ApiAttrData,
	database::base::{client::DatabaseClient, errors::pipeline::GetPipelineError},
	uploader::{errors::UploadAssignError, GotJobKey},
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
pub(super) async fn run_pipeline<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
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
			ApiAttrData::Blob { upload_id } => {
				let res = state.uploader.get_job_object_key(user.id, upload_id).await;
				let key = match res {
					GotJobKey::NoSuchJob => {
						return (
							StatusCode::BAD_REQUEST,
							Json(format!(
								"Invalid input: input {k} references a job that does not exist"
							)),
						)
							.into_response();
					}

					GotJobKey::JobNotDone => {
						return (
							StatusCode::BAD_REQUEST,
							Json(format!(
								"Invalid input: input {k} references a job that is not finished"
							)),
						)
							.into_response();
					}

					GotJobKey::JobIsAssigned => {
						return (
							StatusCode::BAD_REQUEST,
							Json(format!(
								"Invalid input: input {k} references a job that has been assigned to a pipeline"
							)),
						)
							.into_response();
					}

					GotJobKey::HereYouGo(key) => key,
				};

				let res = state
					.uploader
					.assign_job_to_pipeline(user.id, upload_id, &payload.job_id)
					.await;

				match res {
					// This is impossible, we already checked these cases
					Err(UploadAssignError::BadUpload) => unreachable!(),
					Err(UploadAssignError::NotMyUpload) => unreachable!(),

					Ok(()) => Some(AttrData::Blob {
						bucket: (&state.config.edged_objectstore_upload_bucket).into(),
						key,
					}),
				}
			}

			_ => None,
		} {
			converted_input.insert(k, x);
			continue;
		}

		unreachable!("User-provided data {v:?} could not be converted automatically, but was not caught by the manual conversion `match`.")
	}

	let res = state
		.jobqueue_client
		.add_job(
			payload.job_id.as_str().into(),
			user.id,
			&pipe.data,
			&converted_input,
		)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(AddJobError::DbError(error)) => {
			error!(message = "DB error while queueing job", ?error, ?payload.job_id);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(AddJobError::AlreadyExists) => {
			return StatusCode::CONFLICT.into_response();
		}
	};
}
