use axum::{
	extract::{OriginalUri, State},
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	Json,
};
use copper_pipelined::{
	data::{BytesSource, PipeData},
	CopperContext,
};
use copper_storaged::AttrData;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use crate::{pipeline::json::PipelineJson, RouterState};

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct AddJobRequest {
	/// The pipeline
	pub pipeline: PipelineJson,

	/// A unique id for this job
	pub job_id: SmartString<LazyCompact>,

	#[schema(value_type = BTreeMap<String, AttrData>)]
	pub input: BTreeMap<SmartString<LazyCompact>, AttrData>,
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/run",
	responses(
		(status = 200, description = "Job queued successfully"),
		(status = 404, description = "Invalid dataset or pipeline", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
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
	};

	runner.add_job(context, payload.pipeline, &payload.job_id, input);

	return StatusCode::OK.into_response();
}
