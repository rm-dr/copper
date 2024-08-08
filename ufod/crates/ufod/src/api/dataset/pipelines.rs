use std::sync::Arc;

use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_pipeline::labels::PipelineName;
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};
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
	path = "/{dataset_name}/pipelines",
	responses(
		(status = 200, description = "Pipeline names", body = Vec<PipelineInfoShort>),
		(status = 500, description = "Could not load pipeline", body = String),
	),
)]
pub(super) async fn get_all_pipelines(
	Path(dataset_name): Path<String>,
	State(state): State<RouterState>,
) -> Response {
	let dataset = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();

	let context = Arc::new(UFOContext {
		dataset: dataset.clone(),
		blob_fragment_size: 1_000_000,
	});

	let mut out = Vec::new();
	for pipe_name in dataset.all_pipelines().unwrap() {
		let pipe = match dataset.load_pipeline(&pipe_name, context.clone()) {
			Ok(x) => x.unwrap(),
			Err(e) => {
				error!(
					message = "Could not load pipeline",
					pipeline = ?pipe_name,
					error = ?e
				);
				return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")).into_response();
			}
		};

		let input_node_type = pipe.get_node(pipe.input_node_id()).unwrap();

		out.push(PipelineInfoShort {
			name: pipe_name.clone(),
			input_type: PipelineInfoInput::node_to_input_type(input_node_type),
		});
	}

	return Json(out).into_response();
}
