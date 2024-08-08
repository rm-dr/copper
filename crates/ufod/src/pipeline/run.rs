use std::sync::Arc;

use axum::{
	extract::{Path, State},
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use ufo_database::api::UFODatabase;
use ufo_pipeline::{api::PipelineNodeStub, labels::PipelineName};
use ufo_pipeline_nodes::data::{UFOData, UFODataStub};
use utoipa::ToSchema;

use crate::RouterState;

use super::apidata::{ApiData, ApiDataStub};

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct AddJobParams {
	pub input: Vec<ApiData>,

	#[schema(value_type = Option<String>)]
	pub bound_upload_job: Option<SmartString<LazyCompact>>,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub enum AddJobResult {
	Ok, // TODO: return job id
	BadPipeline {
		#[schema(value_type = Option<String>)]
		pipeline: PipelineName,
	},
	InvalidNumberOfArguments {
		got: usize,
		expected: usize,
	},
	InvalidInputType {
		bad_input_idx: usize,
	},
}
/// Get details about a pipeline
#[utoipa::path(
	get,
	path = "/{pipeline_name}/run",
	params(
		("pipeline_name" = String, description = "Pipeline name"),
	),
	responses(
		(status = 200, description = "Pipeline info", body = PipelineInfo),
		(status = 404, description = "There is no pipeline with this name")
	),
)]
pub(super) async fn run_pipeline(
	State(state): State<RouterState>,
	Path(pipeline_name): Path<PipelineName>,
	Json(payload): Json<AddJobParams>,
) -> Response {
	let mut runner = state.runner.lock().await;
	let db = state.database;

	let pipeline = if let Some(pipeline) = db
		.get_pipestore()
		.load_pipeline(&pipeline_name, state.context)
	{
		// TODO: cache pipelines
		pipeline
	} else {
		return Json(AddJobResult::BadPipeline {
			pipeline: pipeline_name,
		})
		.into_response();
	};

	let ctx = runner.get_context();
	let in_node = pipeline.input_node_id();
	let in_node = pipeline.get_node(in_node).unwrap();

	// Check number of arguments
	let expected_inputs = in_node.n_inputs(ctx);
	if expected_inputs != payload.input.len() {
		return Json(AddJobResult::InvalidNumberOfArguments {
			got: payload.input.len(),
			expected: expected_inputs,
		})
		.into_response();
	}

	// Check type of each argument
	for (i, data) in payload.input.iter().enumerate() {
		let t = match data {
			ApiData::None(t) => match t {
				ApiDataStub::Text => UFODataStub::Text,
				ApiDataStub::Blob => UFODataStub::Path,
				ApiDataStub::Integer => UFODataStub::Integer,
				ApiDataStub::PositiveInteger => UFODataStub::PositiveInteger,
				ApiDataStub::Boolean => UFODataStub::Boolean,
				ApiDataStub::Float => UFODataStub::Float,
			},
			ApiData::Text(_) => UFODataStub::Text,
			ApiData::Blob { .. } => UFODataStub::Path,
			ApiData::Integer(_) => UFODataStub::Integer,
			ApiData::PositiveInteger(_) => UFODataStub::PositiveInteger,
			ApiData::Boolean(_) => UFODataStub::Boolean,
			ApiData::Float(_) => UFODataStub::Float,
		};

		if !in_node.input_compatible_with(ctx, 0, t) {
			return Json(AddJobResult::InvalidInputType { bad_input_idx: i }).into_response();
		}
	}

	let mut inputs = Vec::new();
	for i in payload.input {
		let x = match i {
			ApiData::None(t) => UFOData::None(match t {
				ApiDataStub::Text => UFODataStub::Text,
				ApiDataStub::Blob => UFODataStub::Path,
				ApiDataStub::Integer => UFODataStub::Integer,
				ApiDataStub::PositiveInteger => UFODataStub::PositiveInteger,
				ApiDataStub::Boolean => UFODataStub::Boolean,
				ApiDataStub::Float => UFODataStub::Float,
			}),
			ApiData::Text(t) => UFOData::Text(Arc::new(t)),
			ApiData::Blob { file_name } => {
				let j = payload.bound_upload_job.as_ref();

				if j.is_none() {
					panic!();
				}
				let j = j.unwrap();

				if !state
					.uploader
					.has_file_been_finished(j, &file_name)
					.await
					.unwrap()
				{
					panic!("unfinished file!")
				}

				let p = state.uploader.get_job_file_path(j, &file_name).await;

				if let Some(p) = p {
					UFOData::Path(p)
				} else {
					panic!("bad job")
				}
			}
			ApiData::Integer(i) => UFOData::Integer(i),
			ApiData::PositiveInteger(i) => UFOData::PositiveInteger(i),
			ApiData::Boolean(b) => UFOData::Boolean(b),
			ApiData::Float(f) => UFOData::Float(f),
		};

		inputs.push(x);
	}

	let new_id = runner.add_job(Arc::new(pipeline), inputs);

	if let Some(j) = payload.bound_upload_job {
		state
			.uploader
			.bind_job_to_pipeline(&j, new_id)
			.await
			.unwrap();
	}

	return Json(AddJobResult::Ok).into_response();
}
