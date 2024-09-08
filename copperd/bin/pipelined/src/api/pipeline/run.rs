use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use copper_util::mime::MimeType;
use pipelined_node_base::{
	data::{BytesSource, CopperData},
	CopperContext,
};
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use url::Url;
use utoipa::ToSchema;

use crate::{
	pipeline::{json::PipelineJson, spec::PipelineSpec},
	RouterState,
};

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct AddJobParams {
	pub pipeline_name: String,
	pub pipeline_spec: PipelineJson<CopperData>,

	#[schema(value_type = BTreeMap<String, AddJobInput>)]
	pub input: BTreeMap<SmartString<LazyCompact>, AddJobInput>,
}

/// Input that is passed to the pipeline we're running
#[derive(Deserialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum AddJobInput {
	File {
		/// The file to run this pipeline with
		#[schema(value_type = String)]
		file_name: SmartString<LazyCompact>,

		/// The MIME type of this file
		#[schema(value_type = String)]
		mime: MimeType,

		/// A url to this file
		#[schema(value_type = String)]
		url: Url,
	},
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/run",
	responses(
		(status = 200, description = "Job queued successfully", body = PipelineInfo),
		(status = 404, description = "Invalid dataset or pipeline", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn run_pipeline(
	State(state): State<RouterState>,
	Json(payload): Json<AddJobParams>,
) -> Response {
	let mut runner = state.runner.lock().await;

	let mut input = BTreeMap::new();
	for (name, value) in payload.input {
		match value {
			AddJobInput::File {
				file_name,
				mime,
				url,
			} => {
				let path = CopperData::Bytes {
					mime,
					source: BytesSource::Url { url },
				};

				input.insert(name, path);
			}
		}
	}

	let context = CopperContext {
		blob_fragment_size: state.config.blob_fragment_size,
		input,
	};

	let pipe = PipelineSpec::build(
		runner.get_dispatcher(),
		&context,
		&payload.pipeline_name,
		&payload.pipeline_spec,
	)
	.unwrap();

	let new_id = runner.add_job(context, Arc::new(pipe));

	return StatusCode::OK.into_response();
}
