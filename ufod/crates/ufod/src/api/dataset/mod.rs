use crate::{helpers::maindb::dataset::DatasetType, RouterState};
use axum::{
	routing::{get, post},
	Router,
};
use utoipa::OpenApi;

mod itemclass;
mod pipeline;

mod list;
mod new;

use itemclass::{list::*, new::*, new_attr::*};
use pipeline::{get::*, list::*, run::*};

use list::*;
use new::*;

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(
		new_dataset,
		list_datasets,
		list_pipelines,
		get_pipeline,
		run_pipeline,
		new_itemclass,
		list_itemclasses,
		new_itemclass_attr
	),
	components(schemas(
		NewDataset,
		NewDatasetParams,
		NewDatasetError,
		PipelineInfo,
		PipelineInfoShort,
		PipelineInfoInput,
		AddJobResult,
		AddJobParams,
		AddJobInput,
		DatasetInfoShort,
		DatasetType,
		NewItemclassParams,
		ItemclassInfo,
		AttrInfo,
		NewItemclassAttrParams
	))
)]
pub(super) struct DatasetApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		// Datasets
		.route("/", get(list_datasets))
		.route("/", post(new_dataset))
		// Item classes
		.route("/:dataset_name/classes", get(list_itemclasses))
		.route("/:dataset_name/classes", post(new_itemclass))
		.route(
			"/:dataset_name/classes/:class_id/new_attr",
			post(new_itemclass_attr),
		)
		// Pipelines
		.route("/:dataset_name/pipelines", get(list_pipelines))
		.route("/:dataset_name/pipelines/:pipeline_name", get(get_pipeline))
		.route(
			"/:dataset_name/pipelines/:pipeline_name/run",
			post(run_pipeline),
		)
}
