use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::error;
use ufo_ds_core::{api::pipe::Pipestore, errors::PipestoreError};
use ufo_ds_impl::local::LocalDataset;
use ufo_node_base::{
	data::{UFOData, UFODataStub},
	UFOContext,
};
use ufo_pipeline::labels::PipelineName;
use utoipa::{IntoParams, ToSchema};

use crate::RouterState;

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

	/// The input this pipeline takes
	pub inputs: BTreeMap<String, UFODataStub>,

	/// If true, we couldn't load this pipeline successfully.
	pub has_error: bool,
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
		(status = 401, description = "Unauthorized")
	),
)]

pub(super) async fn list_pipelines(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<PipelineListRequest>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

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

	// TODO: this is ugly, fix it!
	// (do while implementing generic datasets)
	let all_pipes =
		match <LocalDataset as Pipestore<UFOData, UFOContext>>::all_pipelines(&dataset).await {
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

	let runner = state.runner.lock().await;

	let mut out = Vec::new();
	for pipe_name in all_pipes {
		let pipe = match dataset
			.load_pipeline(
				runner.get_dispatcher(),
				&UFOContext {
					dataset: dataset.clone(),
					blob_fragment_size: state.config.blob_fragment_size,
				},
				&pipe_name,
			)
			.await
		{
			Ok(x) => {
				// This should never fail---all_pipelines must only return valid names.
				let pipe = x.unwrap();

				PipelineInfoShort {
					name: pipe_name.clone(),
					has_error: false,
					inputs: pipe
						.input_nodes()
						.iter()
						.map(|(n, t)| (n.to_string(), *t))
						.collect(),
				}
			}

			Err(PipestoreError::PipelinePrepareError(_)) => PipelineInfoShort {
				name: pipe_name.clone(),
				inputs: BTreeMap::new(),
				has_error: true,
			},

			Err(e) => {
				error!(
					message = "Could not load pipeline",
					pipeline = ?pipe_name,
					error = ?e
				);
				return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")).into_response();
			}
		};

		out.push(pipe);
	}

	return Json(out).into_response();
}
