use crate::{helpers::maindb::dataset::DatasetType, RouterState};
use axum::{
	routing::{get, post},
	Router,
};
use utoipa::OpenApi;

mod datasets;
mod new_dataset;
mod pipeline;
mod pipelines;
mod run;

use datasets::*;
use new_dataset::*;
use pipeline::*;
use pipelines::*;
use run::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(
		new_dataset,
		get_all_pipelines,
		get_pipeline,
		run_pipeline,
		get_all_datasets
	),
	components(schemas(
		NewDataset,
		NewDatasetParams,
		LocalDatasetMetadataType,
		NewDatasetError,
		PipelineInfo,
		PipelineInfoShort,
		PipelineInfoInput,
		AddJobResult,
		AddJobParams,
		AddJobInput,
		DatasetInfoShort,
		DatasetType,
	))
)]
pub(super) struct DatasetApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		//
		.route("/", get(get_all_datasets))
		.route("/new", post(new_dataset))
		//
		.route("/:dataset_name/pipelines", get(get_all_pipelines))
		.route("/:dataset_name/pipelines/:pipeline_name", get(get_pipeline))
		.route(
			"/:dataset_name/pipelines/:pipeline_name/run",
			post(run_pipeline),
		)
}
