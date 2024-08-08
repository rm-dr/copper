use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::sync::Arc;
use tracing::error;
use ufo_pipeline::labels::PipelineName;
use ufo_pipeline_nodes::{data::UFOData, UFOContext};
use utoipa::ToSchema;

use crate::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(in crate::api) enum AddJobInput {
	File {
		#[schema(value_type = String)]
		upload_job: SmartString<LazyCompact>,

		#[schema(value_type = String)]
		file_name: SmartString<LazyCompact>,
	},
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(in crate::api) struct AddJobParams {
	pub input: AddJobInput,
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
		(status = 200, description = "Job started successfully", body = PipelineInfo),
		(status = 404, description = "Invalid dataset or pipeline", body = String),
		(status = 500, description = "Internal server error", body = String)
	),
)]
pub(in crate::api) async fn run_pipeline(
	State(state): State<RouterState>,
	Path((dataset_name, pipeline_name)): Path<(String, String)>,
	Json(payload): Json<AddJobParams>,
) -> Response {
	let pipeline_name = PipelineName::new(&pipeline_name);

	let mut runner = state.runner.lock().await;

	let dataset = match state.main_db.get_dataset(&dataset_name) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{dataset_name}` does not exist"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = dataset_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name: {e}"),
			)
				.into_response();
		}
	};

	let context = Arc::new(UFOContext {
		dataset: dataset.clone(),
		blob_fragment_size: 1_000_000,
	});

	let pipe = match dataset.load_pipeline(&pipeline_name, context.clone()) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!(
					"Dataset `{dataset_name}` does not have a pipeline named `{pipeline_name}`"
				),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get pipeline by name",
				dataset = dataset_name,
				pipeline_name = ?pipeline_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get pipeline by name: {e}"),
			)
				.into_response();
		}
	};

	let bound_upload_job: Option<SmartString<LazyCompact>>;
	let inputs = match payload.input {
		AddJobInput::File {
			upload_job,
			file_name,
		} => {
			bound_upload_job = Some(upload_job.clone());

			match state
				.uploader
				.has_file_been_finished(&upload_job, &file_name)
				.await
			{
				Some(true) => {}
				Some(false) => {
					return (
						StatusCode::BAD_REQUEST,
						format!("File `{file_name}` has not finished uploading"),
					)
						.into_response();
				}
				None => {
					return (StatusCode::BAD_REQUEST, format!("Bad upload job or file"))
						.into_response();
				}
			};

			let path = state
				.uploader
				.get_job_file_path(&upload_job, &file_name)
				.await;

			let path = if let Some(path) = path {
				UFOData::Path(path)
			} else {
				// This shouldn't ever happen, since we checked for existence above
				error!(
					message = "Could not get upload file path",
					upload_job = ?upload_job,
					file_name = ?file_name
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get upload file path"),
				)
					.into_response();
			};

			vec![path]
		}
	};

	let new_id = runner.add_job(context, Arc::new(pipe), inputs);

	if let Some(j) = bound_upload_job {
		match state.uploader.bind_job_to_pipeline(&j, new_id).await {
			Ok(()) => {}
			Err(e) => {
				error!(
					message = "Could not bind upload job",
					upload_job = ?j,
					file_name = ?new_id,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not bind upload job: {e:?}"),
				)
					.into_response();
			}
		};
	}

	return StatusCode::OK.into_response();
}
