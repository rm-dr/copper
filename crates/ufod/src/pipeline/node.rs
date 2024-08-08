use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use ufo_database::api::UFODatabase;
use ufo_pipeline::{
	api::PipelineNodeStub,
	labels::{PipelineName, PipelineNodeID},
};
use ufo_pipeline_nodes::data::UFODataStub;
use utoipa::ToSchema;

use crate::RouterState;

use super::apidata::ApiDataStub;

/// A pipeline node specification
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct NodeInfo {
	/// This node's name
	#[schema(value_type = String)]
	pub name: PipelineNodeID,

	/// A list of types each of this node's inputs accepts
	pub inputs: Vec<Vec<ApiDataStub>>,
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
		.get_pipestore()
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

	let inputs = (0..node.n_inputs(&state.context))
		.map(|i| {
			UFODataStub::iter_all()
				.filter(|stub| node.input_compatible_with(&state.context, i, **stub))
				.map(|x| match x {
					UFODataStub::Text => ApiDataStub::Text,
					UFODataStub::Path => ApiDataStub::Blob,
					UFODataStub::Binary => todo!(),
					UFODataStub::Blob => todo!(),
					UFODataStub::Integer => ApiDataStub::Integer,
					UFODataStub::PositiveInteger => ApiDataStub::PositiveInteger,
					UFODataStub::Boolean => ApiDataStub::Boolean,
					UFODataStub::Float => ApiDataStub::Float,
					UFODataStub::Hash { .. } => todo!(),
					UFODataStub::Reference { .. } => todo!(),
				})
				.collect()
		})
		.collect::<Vec<_>>();

	return (
		StatusCode::OK,
		Json(NodeInfo {
			name: node_id,
			inputs,
		}),
	)
		.into_response();
}
