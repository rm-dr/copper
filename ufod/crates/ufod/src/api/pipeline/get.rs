use std::sync::Arc;

use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_pipeline::labels::{PipelineName, PipelineNodeID};
use ufo_pipeline_nodes::UFOContext;
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
		(status = 404, description = "There is no such pipeline or database", body=String),
		(status = 500, description = "Internal server error", body=String)
	),
)]
pub(in crate::api) async fn get_pipeline(
	State(state): State<RouterState>,
	Query(query): Query<PipelineSelect>,
) -> Response {
	let pipeline_name = PipelineName::new(&query.pipeline);

	let dataset = match state.main_db.get_dataset(&query.dataset) {
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

	let context = Arc::new(UFOContext {
		dataset: dataset.clone(),
		// TODO: config & publish
		blob_fragment_size: 1_000_000,
	});

	let pipe = match dataset.load_pipeline(&pipeline_name, context) {
		Ok(Some(x)) => x,
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

	let node_ids = pipe.iter_node_ids().cloned().collect::<Vec<_>>();
	let input_node_type = pipe.get_node(pipe.input_node_id()).unwrap();

	return (
		StatusCode::OK,
		Json(Some(PipelineInfo {
			short: PipelineInfoShort {
				name: pipeline_name,
				input_type: PipelineInfoInput::node_to_input_type(input_node_type),
			},
			nodes: node_ids,
			input_node: pipe.input_node_id().clone(),
		})),
	)
		.into_response();
}