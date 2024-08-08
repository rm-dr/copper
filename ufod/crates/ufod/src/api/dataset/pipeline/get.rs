use std::sync::Arc;

use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use ufo_pipeline::labels::{PipelineName, PipelineNodeID};
use ufo_pipeline_nodes::UFOContext;
use utoipa::ToSchema;

use super::list::{PipelineInfoInput, PipelineInfoShort};
use crate::RouterState;

/// A pipeline specification
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(in crate::api) struct PipelineInfo {
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
	path = "/{dataset_name}/pipelines/{pipeline_name}",
	tag = "Pipeline",
	params(
		("dataset_name" = String, description = "Dataset name"),
		("pipeline_name" = String, description = "Pipeline name"),
	),
	responses(
		(status = 200, description = "Pipeline info", body = PipelineInfo),
		(status = 404, description = "There is no pipeline with this name")
	),
)]
pub(in crate::api) async fn get_pipeline(
	Path((dataset_name, pipeline_name)): Path<(String, String)>,
	State(state): State<RouterState>,
) -> Response {
	let pipeline_name = PipelineName::new(&pipeline_name);
	let dataset = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();

	let context = Arc::new(UFOContext {
		dataset: dataset.clone(),
		blob_fragment_size: 1_000_000,
	});

	let pipe = if let Some(pipe) = state
		.main_db
		.get_dataset(&dataset_name)
		.unwrap()
		.unwrap()
		.load_pipeline(&pipeline_name, context)
		.unwrap()
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
