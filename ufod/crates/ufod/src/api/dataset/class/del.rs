use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

// Validate confirmation here, just in case.
// It's really easy to send an empty request, these
// required parameters make it harder to accidentally
// delete valuable data.

/// Confirmation info for class deletion
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(in crate::api) struct DeleteClassConfirmation {
	/// The dataset to delete from.
	/// Should match request.
	pub dataset_name: String,

	/// The name of the class to delete.
	/// Should match request.
	pub class_name: String,
}

/// Delete a class and all data associated with it
#[utoipa::path(
	delete,
	path = "/{dataset_name}/classes/{class_name}",
	tag = "Itemclass",
	params(
		("dataset_name" = String, description = "Dataset name"),
		("class_name" = String, description = "New class name")
	),
	responses(
		(status = 200, description = "Successfully deleted this class"),
		(status = 400, description = "Invalid request", body=String),
		(status = 404, description = "This dataset or  class doesn't exist", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(in crate::api) async fn del_class(
	Path((dataset_name, class_name)): Path<(String, String)>,
	State(state): State<RouterState>,
	Json(confirmation): Json<DeleteClassConfirmation>,
) -> Response {
	if confirmation.class_name != class_name {
		return (
			StatusCode::BAD_REQUEST,
			format!("Confirmation does not match class name"),
		)
			.into_response();
	}

	if confirmation.dataset_name != dataset_name {
		return (
			StatusCode::BAD_REQUEST,
			format!("Confirmation does not match dataset name"),
		)
			.into_response();
	}

	let dataset = match state.main_db.get_dataset(&dataset_name) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{dataset_name}` does not exist"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = dataset_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name: {e}"),
			)
				.into_response();
		}
	};

	let class_handle = match dataset.get_class(&class_name) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{class_name}` does not exist"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class by name",
				dataset = dataset_name,
				class_name = ?class_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class by name: {e}"),
			)
				.into_response();
		}
	};

	let res = dataset.del_class(class_handle);

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(e) => {
			error!(
				message = "Could not delete class",
				dataset = dataset_name,
				class_name = class_name,
				class_handle = ?class_handle,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete class: {e}"),
			)
				.into_response();
		}
	}
}
