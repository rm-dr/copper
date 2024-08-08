use std::sync::Arc;

use crate::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_pipeline::labels::PipelineName;
use ufo_pipeline_nodes::{nodetype::UFONodeType, UFOContext};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Serialize, ToSchema, Debug, IntoParams)]
pub(super) struct PipelineListRequest {
	/// Which dataset's pipelines we want to list
	pub dataset: String,
}

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
	path = "/list",
	params(PipelineListRequest),
	responses(
		(status = 200, description = "Pipeline info", body = Vec<PipelineInfoShort>),
		(status = 404, description = "This dataset doesn't exist", body = String),
		(status = 500, description = "Could not load pipeline", body = String),
	),
)]

pub(super) async fn list_pipelines(
	State(state): State<RouterState>,
	Query(query): Query<PipelineListRequest>,
) -> Response {
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
		blob_fragment_size: 1_000_000,
	});

	let all_pipes = match dataset.all_pipelines() {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not list pipelines",
				dataset = query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not list pipelines: {e}"),
			)
				.into_response();
		}
	};

	let mut out = Vec::new();
	for pipe_name in all_pipes {
		let pipe = match dataset.load_pipeline(&pipe_name, context.clone()) {
			// This should never fail---all_pipelines must only return valid names.
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

		// Same thing here---this should not be none.
		let input_node_type = pipe.get_node(pipe.input_node_id()).unwrap();

		out.push(PipelineInfoShort {
			name: pipe_name.clone(),
			input_type: PipelineInfoInput::node_to_input_type(input_node_type),
		});
	}

	return Json(out).into_response();
}
