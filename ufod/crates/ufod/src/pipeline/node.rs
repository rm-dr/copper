use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use ufo_pipeline::labels::{PipelineName, PipelineNodeID};
use ufo_pipeline_nodes::nodetype::UFONodeType;
use utoipa::ToSchema;

use crate::RouterState;

/// A pipeline node specification
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NodeInfo {
	/// This node's name
	#[schema(value_type = String)]
	pub name: PipelineNodeID,

	/// This node's type
	pub node_type: UFONodeType,
}

/// Get details about a node in a pipeline
#[utoipa::path(
	get,
	path = "/{pipeline_name}/{node_id}",
	params(
		("pipeline_name", description = "Pipeline name"),
		("node_id", description = "Node id"),
	),
	responses(
		(status = 200, description = "Node info", body = NodeInfo),
		(status = 404, description = "There is either no pipeline with the given name, or this pipeline has no such node")
	),
)]
pub(super) async fn get_pipeline_node(
	Path((pipeline_name, node_id)): Path<(String, String)>,
	State(state): State<RouterState>,
) -> Response {
	let pipeline_name = PipelineName::new(&pipeline_name);
	let node_id = PipelineNodeID::new(&node_id);

	let pipe = if let Some(pipe) = state
		.database
		.load_pipeline(&pipeline_name, state.context.clone())
	{
		pipe
	} else {
		return StatusCode::NOT_FOUND.into_response();
	};

	let node = if let Some(node) = pipe.get_node(&node_id) {
		node
	} else {
		return StatusCode::NOT_FOUND.into_response();
	};

	return (
		StatusCode::OK,
		Json(NodeInfo {
			name: node_id,
			node_type: node.clone(),
		}),
	)
		.into_response();
}
