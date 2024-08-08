use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::sync::Arc;
use tracing::error;
use ufo_ds_core::{api::pipe::Pipestore, errors::PipestoreError};
use ufo_pipeline::labels::PipelineName;
use ufo_pipeline_nodes::{data::UFOData, UFOContext};
use utoipa::ToSchema;

use super::PipelineSelect;
use crate::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct AddJobParams {
	#[serde(flatten)]
	pub pipe: PipelineSelect,

	pub input: AddJobInput,
}

/// Input that is passed to the pipeline we're running
#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(tag = "type")]
pub(super) enum AddJobInput {
	File {
		/// The upload job we uploaded a file to
		#[schema(value_type = String)]
		upload_job: SmartString<LazyCompact>,

		/// The file to run this pipeline with
		#[schema(value_type = String)]
		file_name: SmartString<LazyCompact>,
	},
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/run",
	responses(
		(status = 200, description = "Job started successfully", body = PipelineInfo),
		(status = 404, description = "Invalid dataset or pipeline", body = String),
		(status = 500, description = "Internal server error", body = String)
	),
)]
pub(super) async fn run_pipeline(
	State(state): State<RouterState>,
	Json(payload): Json<AddJobParams>,
) -> Response {
	let pipeline_name = PipelineName::new(&payload.pipe.pipeline);

	let mut runner = state.runner.lock().await;

	let dataset = match state
		.main_db
		.dataset
		.get_dataset(&payload.pipe.dataset)
		.await
	{
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", payload.pipe.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = payload.pipe.dataset,
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
		blob_fragment_size: state.config.blob_fragment_size,
	});

	let pipe = match dataset.load_pipeline(&pipeline_name, context.clone()) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!(
					"Dataset `{}` does not have a pipeline named `{pipeline_name}`",
					payload.pipe.dataset
				),
			)
				.into_response()
		}

		Err(PipestoreError::PipelinePrepareError(_)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Cannot run invalid pipeline"),
			)
				.into_response();
		}

		Err(e) => {
			error!(
				message = "Could not get pipeline by name",
				dataset = payload.pipe.dataset,
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
