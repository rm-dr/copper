use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::errors::MetastoreError;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(in crate::api) struct NewItemclassParams {
	/// The new item class name
	pub name: String,
}

/// Create a new itemclass
#[utoipa::path(
	post,
	path = "/{dataset_name}/classes",
	tag = "Itemclass",
	params(
		("dataset_name" = String, description = "Dataset name")
	),
	responses(
		(status = 200, description = "Successfully created new item class"),
		(status = 400, description = "Could not create new item class, bad parameters", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(in crate::api) async fn new_itemclass(
	Path(dataset_name): Path<String>,
	State(state): State<RouterState>,
	Json(new_params): Json<NewItemclassParams>,
) -> Response {
	let dataset = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();
	let res = dataset.add_class(&new_params.name);

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateClassName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Class name `{x}` already exists"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not create new item class",
				dataset = dataset_name,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")).into_response();
		}
	}
}
