use crate::RouterState;
use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use ufo_database::api::UFODatabase;
use ufo_pipeline::labels::PipelineName;
use ufo_pipeline_nodes::nodetype::UFONodeType;
use utoipa::ToSchema;

/// A pipeline specification
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct PipelineInfoShort {
	/// This pipeline's name
	#[schema(value_type = String)]
	pub name: PipelineName,

	pub input_type: PipelineInfoInput,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) enum PipelineInfoInput {
	/// This pipeline's input may not be provided through the api
	None,

	/// This pipeline consumes a file
	File,
}

impl PipelineInfoInput {
	pub(super) fn node_to_input_type(input_node_type: &UFONodeType) -> Self {
		// This MUST match the decode implementation in `./run.rs`
		match input_node_type {
			UFONodeType::File => PipelineInfoInput::File,
			_ => PipelineInfoInput::None,
		}
	}
}

/// Get all pipelines
#[utoipa::path(
	get,
	path = "",
	responses(
		(status = 200, description = "Pipeline names", body = Vec<PipelineInfoShort>),
	),
)]
pub(super) async fn get_all_pipelines(State(state): State<RouterState>) -> impl IntoResponse {
	return Json(
		state
			.database
			.get_pipestore()
			.all_pipelines()
			.iter()
			.map(|pipe_name| {
				let pipe = state
					.database
					.get_pipestore()
					.load_pipeline(&pipe_name, state.context.clone())
					.unwrap();
				let input_node_type = pipe.get_node(pipe.input_node_id()).unwrap();

				PipelineInfoShort {
					name: pipe_name.clone(),
					input_type: PipelineInfoInput::node_to_input_type(input_node_type),
				}
			})
			.collect::<Vec<_>>(),
	);
}
