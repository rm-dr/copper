use std::sync::Arc;

use axum::{
	extract::{Path, State},
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::labels::PipelineName;
use ufo_pipeline_nodes::{data::UFOData, UFOContext};
use utoipa::ToSchema;

use crate::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum AddJobInput {
	File {
		#[schema(value_type = String)]
		upload_job: SmartString<LazyCompact>,

		#[schema(value_type = String)]
		file_name: SmartString<LazyCompact>,
	},
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct AddJobParams {
	pub input: AddJobInput,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum AddJobResult {
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

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/{dataset_name}/pipelines/{pipeline_name}/run",
	params(
		("dataset_name" = String, description = "Dataset name"),
		("pipeline_name" = String, description = "Pipeline name"),
	),
	responses(
		(status = 200, description = "Pipeline info", body = PipelineInfo),
		(status = 404, description = "There is no pipeline with this name")
	),
)]
pub(super) async fn run_pipeline(
	State(state): State<RouterState>,
	Path((dataset_name, pipeline_name)): Path<(String, String)>,
	Json(payload): Json<AddJobParams>,
) -> Response {
	let pipeline_name = PipelineName::new(&pipeline_name);

	let mut runner = state.runner.lock().await;
	let ds = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();

	let context = Arc::new(UFOContext {
		dataset: ds.clone(),
		blob_fragment_size: 1_000_000,
	});

	let pipeline =
		if let Some(pipeline) = ds.load_pipeline(&pipeline_name, context.clone()).unwrap() {
			// TODO: cache pipelines
			pipeline
		} else {
			return Json(AddJobResult::BadPipeline {
				pipeline: pipeline_name,
			})
			.into_response();
		};

	let bound_upload_job: Option<SmartString<LazyCompact>>;
	let inputs = match payload.input {
		AddJobInput::File {
			upload_job,
			file_name,
		} => {
			bound_upload_job = Some(upload_job.clone());

			if !state
				.uploader
				.has_file_been_finished(&upload_job, &file_name)
				.await
				.unwrap()
			{
				panic!("unfinished file!")
			}

			let path = state
				.uploader
				.get_job_file_path(&upload_job, &file_name)
				.await;

			let path = if let Some(path) = path {
				UFOData::Path(path)
			} else {
				panic!("bad job")
			};

			vec![path]
		}
	};

	let new_id = runner.add_job(context, Arc::new(pipeline), inputs);

	if let Some(j) = bound_upload_job {
		state
			.uploader
			.bind_job_to_pipeline(&j, new_id)
			.await
			.unwrap();
	}

	return Json(AddJobResult::Ok).into_response();
}
