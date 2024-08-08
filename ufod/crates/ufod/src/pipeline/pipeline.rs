use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use ufo_database::api::UFODatabase;
use ufo_pipeline::labels::{PipelineName, PipelineNodeID};
use utoipa::ToSchema;

use super::{PipelineInfoInput, PipelineInfoShort};
use crate::RouterState;

/// A pipeline specification
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct PipelineInfo {
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
	path = "/{pipeline_name}",
	params(
		("pipeline_name" = String, description = "Pipeline name"),
	),
	responses(
		(status = 200, description = "Pipeline info", body = PipelineInfo),
		(status = 404, description = "There is no pipeline with this name")
	),
)]
pub(super) async fn get_pipeline(
	Path(pipeline_name): Path<PipelineName>,
	State(state): State<RouterState>,
) -> Response {
	let pipe = if let Some(pipe) = state
		.database
		.get_pipestore()
		.load_pipeline(&pipeline_name, state.context)
	{
		pipe
	} else {
		return StatusCode::NOT_FOUND.into_response();
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
