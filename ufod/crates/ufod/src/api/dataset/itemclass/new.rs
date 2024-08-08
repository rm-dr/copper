use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use tracing::error;
use ufo_ds_core::errors::MetastoreError;

use crate::api::RouterState;

/// Create a new itemclass
#[utoipa::path(
	post,
	path = "/{dataset_name}/classes/{class_name}",
	tag = "Itemclass",
	params(
		("dataset_name" = String, description = "Dataset name"),
		("class_name" = String, description = "New class name")
	),
	responses(
		(status = 200, description = "Successfully created new item class"),
		(status = 400, description = "Could not create new item class, bad parameters", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(in crate::api) async fn new_itemclass(
	Path((dataset_name, class_name)): Path<(String, String)>,
	State(state): State<RouterState>,
) -> Response {
	let dataset = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();
	let res = dataset.add_class(&class_name);

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
