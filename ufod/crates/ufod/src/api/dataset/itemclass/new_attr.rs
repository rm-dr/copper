use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::{
	api::meta::AttributeOptions, data::MetastoreDataStub, errors::MetastoreError,
	handles::ClassHandle,
};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(in crate::api) struct NewItemclassAttrParams {
	/// The new attribute's name
	pub name: String,

	/// The new attribute's data type
	pub data_type: MetastoreDataStub,

	/// Options for this new attribute
	pub options: AttributeOptions,
}

/// Create a new attribute in this itemclass
#[utoipa::path(
	post,
	path = "/{dataset_name}/classes/{class_id}/new_attr",
	tag = "Itemclass",
	params(
		("dataset_name" = String, description = "Dataset name"),
		("class_id" = u32, description = "Itemclass id"),
	),
	responses(
		(status = 200, description = "Successfully created new attribute"),
		(status = 400, description = "Could not create new attribute, bad parameters", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(in crate::api) async fn new_itemclass_attr(
	Path((dataset_name, class_id)): Path<(String, u32)>,
	State(state): State<RouterState>,
	Json(new_params): Json<NewItemclassAttrParams>,
) -> Response {
	let class_id: ClassHandle = class_id.into();

	let dataset = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();
	let res = dataset.add_attr(
		class_id,
		&new_params.name,
		new_params.data_type,
		new_params.options,
	);

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateAttrName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Attribute `{x}` already exists on class `{class_id:?}`"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not create new attribute",
				dataset = dataset_name,
				item_class = ?class_id,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")).into_response();
		}
	}
}
