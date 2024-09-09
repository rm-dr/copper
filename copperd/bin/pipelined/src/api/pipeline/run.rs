use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use copper_pipelined::{data::PipeData, CopperContext};
use copper_storaged::{client::ReqwestStoragedClient, AttrData};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tracing::error;
use utoipa::ToSchema;

use crate::{
	pipeline::{json::PipelineJson, spec::PipelineSpec},
	RouterState,
};

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct AddJobRequest {
	pub pipeline_name: String,
	pub pipeline_spec: PipelineJson<PipeData>,

	#[schema(value_type = BTreeMap<String, AttrData>)]
	pub input: BTreeMap<SmartString<LazyCompact>, AttrData>,
}

/// Input that is passed to the pipeline we're running
#[derive(Serialize, ToSchema, Debug)]
pub(super) struct AddJobResponse {
	new_job_id: u128,
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/run",
	responses(
		(status = 200, description = "Job queued successfully", body = AddJobResponse),
		(status = 404, description = "Invalid dataset or pipeline", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn run_pipeline(
	State(state): State<RouterState>,
	Json(payload): Json<AddJobRequest>,
) -> Response {
	let runner = state.runner.lock().await;

	let mut input = BTreeMap::new();
	for (name, value) in payload.input {
		// TODO: handle special cases (unwrap)
		input.insert(name, PipeData::try_from(value).unwrap());
	}

	let context = CopperContext {
		blob_fragment_size: state.config.pipelined_blob_fragment_size,
		storaged_client: Arc::new(
			ReqwestStoragedClient::new(state.config.pipelined_storaged_addr.clone()).unwrap(),
		),
		input,
	};

	let pipe = PipelineSpec::build(
		runner.get_dispatcher(),
		&payload.pipeline_name,
		&payload.pipeline_spec,
	)
	.unwrap();

	// Prevent a deadlock with below code
	drop(runner);

	// Allow `add_job` to block
	let x = state.runner.clone();
	let new_job_result = tokio::task::spawn_blocking(move || {
		let mut y = block_on(x.lock());
		y.add_job(context, Arc::new(pipe))
	})
	.await;

	match new_job_result {
		Ok(Ok(new_job_id)) => {
			return (StatusCode::OK, Json(AddJobResponse { new_job_id })).into_response();
		}

		Ok(Err(e)) => {
			return (
				StatusCode::BAD_REQUEST,
				Json(format!("Could not create job: {e:?}")),
			)
				.into_response()
		}

		Err(e) => {
			error!(message = "Join error while starting job", error = ?e);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	}
}
