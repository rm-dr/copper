use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	routing::get,
	Json, Router,
};
use ufo_api::{
	data::ApiDataStub,
	pipeline::{NodeInfo, PipelineInfo},
};
use ufo_database::api::UFODatabase;
use ufo_pipeline::{
	api::PipelineNodeStub,
	labels::{PipelineLabel, PipelineNodeLabel},
};
use ufo_pipeline_nodes::data::UFODataStub;

pub fn router() -> Router<RouterState> {
	Router::new()
		.route("/", get(get_all_pipelines))
		.route("/:pipeline_name", get(get_pipeline))
		.route("/:pipeline_name/:node_name", get(get_pipeline_node))
}

/// Get all pipeline names
async fn get_all_pipelines(State(state): State<RouterState>) -> impl IntoResponse {
	return Json(state.database.get_pipestore().all_pipelines().clone());
}

/// Get details about one pipeline
async fn get_pipeline(
	Path(pipeline_name): Path<PipelineLabel>,
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

	let nodes = pipe.iter_node_labels().cloned().collect::<Vec<_>>();

	return (
		StatusCode::OK,
		Json(Some(PipelineInfo {
			name: pipeline_name,
			nodes,
			input_node: pipe.input_node_label().clone(),
			output_node: pipe.output_node_label().clone(),
		})),
	)
		.into_response();
}

/// Get details about a node in one pipeline
async fn get_pipeline_node(
	Path((pipeline_name, node_name)): Path<(PipelineLabel, PipelineNodeLabel)>,
	State(state): State<RouterState>,
) -> Response {
	let pipe = if let Some(pipe) = state
		.database
		.get_pipestore()
		.load_pipeline(&pipeline_name, state.context.clone())
	{
		pipe
	} else {
		return StatusCode::NOT_FOUND.into_response();
	};

	let node = if let Some(node) = pipe.get_node(&node_name) {
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
		Json(Some(NodeInfo {
			name: node_name,
			inputs,
		})),
	)
		.into_response();
}
