use axum::{
	extract::{OriginalUri, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_pipelined::{
	data::{BytesSource, PipeData},
	json::PipelineJson,
	CopperContext,
};
use copper_storaged::{AttrData, Transaction, UserId};
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tokio::sync::Mutex;
use utoipa::ToSchema;

use crate::{pipeline::runner::AddJobError, RouterState};

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct AddJobRequest {
	/// The pipeline
	pub pipeline: PipelineJson,

	/// A unique id for this job
	#[schema(value_type = String)]
	pub job_id: SmartString<LazyCompact>,

	/// The input to pass to this pipeline
	#[schema(value_type = BTreeMap<String, AttrData>)]
	pub input: BTreeMap<SmartString<LazyCompact>, AttrData>,

	/// The user to run this pipeline as
	#[schema(value_type = i64)]
	pub as_user: UserId,
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/run",
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
pub(super) async fn run_pipeline(
	headers: HeaderMap,
	OriginalUri(uri): OriginalUri,
	State(state): State<RouterState>,
	Json(payload): Json<AddJobRequest>,
) -> Response {
	if !state.config.header_has_valid_auth(&uri, &headers) {
		return StatusCode::UNAUTHORIZED.into_response();
	};

	let mut runner = state.runner.lock().await;

	let mut input = BTreeMap::new();
	for (name, value) in payload.input {
		match value {
			AttrData::Blob { object_key } => input.insert(
				name,
				PipeData::Blob {
					source: BytesSource::S3 { key: object_key },
				},
			),

			// This should never fail, we handle all special cases above
			_ => input.insert(name, PipeData::try_from(value).unwrap()),
		};
	}

	let context = CopperContext {
		blob_fragment_size: state.config.pipelined_blob_fragment_size,
		stream_channel_capacity: state.config.pipelined_stream_channel_size,
		objectstore_client: state.objectstore_client.clone(),
		storaged_client: state.storaged_client.clone(),
		job_id: payload.job_id.clone(),
		transaction: Mutex::new(Transaction::new()),
		run_by_user: payload.as_user,
	};

	return match runner.add_job(
		context,
		payload.pipeline,
		&payload.job_id,
		payload.as_user,
		input,
	) {
		Ok(()) => StatusCode::OK.into_response(),
		Err(AddJobError::AlreadyExists) => StatusCode::CONFLICT.into_response(),
		Err(AddJobError::QueueFull) => StatusCode::TOO_MANY_REQUESTS.into_response(),
	};
}
