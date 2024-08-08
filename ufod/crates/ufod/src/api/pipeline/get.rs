use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::{api::pipe::Pipestore, errors::PipestoreError};
use ufo_node_base::UFOContext;
use ufo_pipeline::labels::{PipelineName, PipelineNodeID};
use utoipa::ToSchema;

use super::list::{PipelineInfoInput, PipelineInfoShort};
use super::PipelineSelect;
use crate::RouterState;

/// A pipeline specification
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct PipelineInfo {
	#[serde(flatten)]
	pub short: PipelineInfoShort,

	/// A list of nodes in this pipeline
	#[schema(value_type = Vec<String>)]
	pub nodes: Vec<PipelineNodeID>,

	/// This pipeline's input node
	#[schema(value_type = String)]
	pub input_node: PipelineNodeID,
}

/// Get details about a pipeline
#[utoipa::path(
	get,
	path = "/get",
	params(PipelineSelect),
	responses(
		(status = 200, description = "Pipeline info", body = PipelineInfo),
		(status = 404, description = "There is no such pipeline or database", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn get_pipeline(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<PipelineSelect>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	let pipeline_name = PipelineName::new(&query.pipeline);

	let dataset = match state.main_db.dataset.get_dataset(&query.dataset).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", query.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name: {e}"),
			)
				.into_response();
		}
	};

	let runner = state.runner.lock().await;

	// TODO: clean up.
	// We shouldn't need to load a pipeline to get its info
	match dataset
		.load_pipeline(
			runner.get_dispatcher(),
			&UFOContext {
				dataset: dataset.clone(),
				blob_fragment_size: state.config.blob_fragment_size,
			},
			&pipeline_name,
		)
		.await
	{
		Ok(Some(pipe)) => {
			let node_ids = pipe.iter_node_ids().cloned().collect::<Vec<_>>();
			let input_node_type = pipe.get_node(pipe.input_node_id()).unwrap();

			return (
				StatusCode::OK,
				Json(Some(PipelineInfo {
					short: PipelineInfoShort {
						name: pipeline_name,
						input_type: match &input_node_type.node_type[..] {
							// TODO: rework this
							"file" => PipelineInfoInput::File,
							_ => PipelineInfoInput::None,
						},
						has_error: false,
					},
					nodes: node_ids,
					input_node: pipe.input_node_id().clone(),
				})),
			)
				.into_response();
		}
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!(
					"Dataset `{}` does not have a pipeline named `{pipeline_name}`",
					query.dataset
				),
			)
				.into_response()
		}

		Err(PipestoreError::PipelinePrepareError(_)) => {
			return (
				StatusCode::OK,
				Json(Some(PipelineInfo {
					short: PipelineInfoShort {
						name: pipeline_name,
						input_type: PipelineInfoInput::None,
						has_error: false,
					},
					nodes: vec![],
					input_node: PipelineNodeID::new("INVALID"),
				})),
			)
				.into_response();
		}

		Err(e) => {
			error!(
				message = "Could not get pipeline by name",
				dataset = query.dataset,
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
}
