use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use tracing::{error, warn};
use ufo_ds_core::{api::pipe::Pipestore, errors::PipestoreError};
use ufo_node_base::{
	data::{BytesSource, CopperData},
	CopperContext,
};
use ufo_pipeline::labels::PipelineName;
use ufo_util::mime::MimeType;
use utoipa::ToSchema;

use super::PipelineSelect;
use crate::RouterState;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct AddJobParams {
	#[serde(flatten)]
	pub pipe: PipelineSelect,

	#[schema(value_type = BTreeMap<String, AddJobInput>)]
	pub input: BTreeMap<SmartString<LazyCompact>, AddJobInput>,
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
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
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

	let mut bound_upload_jobs = Vec::new();
	let mut input = BTreeMap::new();
	for (name, value) in payload.input {
		match value {
			AddJobInput::File {
				upload_job,
				file_name,
			} => {
				bound_upload_jobs.push(upload_job.clone());

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
						return (StatusCode::BAD_REQUEST, "Bad upload job or file").into_response();
					}
				};

				let path = state
					.uploader
					.get_job_file_path(&upload_job, &file_name)
					.await;

				let path = if let Some(path) = path {
					CopperData::Bytes {
						mime: {
							path.extension()
								.map(|x| {
									MimeType::from_extension(x.to_str().unwrap())
										.unwrap_or(MimeType::Blob)
								})
								.unwrap_or(MimeType::Blob)
						},
						source: BytesSource::File { path },
					}
				} else {
					// This shouldn't ever happen, since we checked for existence above
					error!(
						message = "Could not get upload file path",
						upload_job = ?upload_job,
						file_name = ?file_name
					);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						"Could not get upload file path",
					)
						.into_response();
				};

				input.insert(name, path);
			}
		}
	}

	let context = CopperContext {
		dataset: dataset.clone(),
		blob_fragment_size: state.config.pipeline.blob_fragment_size,
		input,
	};

	let pipe = match dataset
		.load_pipeline(
			runner.get_dispatcher(),
			&context, // Unused when building pipelines
			&pipeline_name,
		)
		.await
	{
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

		Err(PipestoreError::PipelinePrepareError(e)) => {
			warn!(
				message = "Cannot run invalid pipeline",
				pipeline = ?pipeline_name,
				dataset = payload.pipe.dataset,
				error = ?e
			);
			return (StatusCode::BAD_REQUEST, "Cannot run invalid pipeline").into_response();
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

	let new_id = runner.add_job(context, Arc::new(pipe));

	for j in bound_upload_jobs {
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
