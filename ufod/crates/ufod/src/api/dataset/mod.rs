use crate::{helpers::maindb::dataset::DatasetType, RouterState};
use axum::{
	routing::{delete, get, post},
	Router,
};
use utoipa::OpenApi;

mod class;
mod pipeline;

mod del;
mod list;
mod new;

use class::{del::*, del_attr::*, list::*, new::*, new_attr::*};
use pipeline::{get::*, list::*, run::*};

use del::*;
use list::*;
use new::*;

// TODO: rename all "itemclass" to "class"
// (or maybe something else?)

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(
		new_dataset,
		list_datasets,
		list_pipelines,
		get_pipeline,
		run_pipeline,
		new_class,
		list_classes,
		new_class_attr,
		del_class,
		del_class_attr,
		del_dataset
	),
	components(schemas(
		NewDatasetParams,
		PipelineInfo,
		PipelineInfoShort,
		PipelineInfoInput,
		AddJobParams,
		AddJobInput,
		DatasetInfoShort,
		DatasetType,
		ClassInfo,
		AttrInfo,
		NewClassAttrParams,
		DeleteAttrConfirmation,
		DeleteClassConfirmation,
		DeleteDatasetConfirmation
	))
)]
pub(super) struct DatasetApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new()
		// Datasets
		.route("/", get(list_datasets))
		.route("/:dataset_name", post(new_dataset))
		.route("/:dataset_name", delete(del_dataset))
		// Item classes
		.route("/:dataset_name/classes", get(list_classes))
		.route("/:dataset_name/classes/:class_name", post(new_class))
		.route("/:dataset_name/classes/:class_name", delete(del_class))
		.route(
			"/:dataset_name/classes/:class_name/attrs/:attr_name",
			post(new_class_attr),
		)
		.route(
			"/:dataset_name/classes/:class_name/attrs/:attr_name",
			delete(del_class_attr),
		)
		// Pipelines
		.route("/:dataset_name/pipelines", get(list_pipelines))
		.route("/:dataset_name/pipelines/:pipeline_name", get(get_pipeline))
		.route(
			"/:dataset_name/pipelines/:pipeline_name/run",
			post(run_pipeline),
		)
}
