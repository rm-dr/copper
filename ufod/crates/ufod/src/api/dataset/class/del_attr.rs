use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::handles::ClassHandle;
use utoipa::ToSchema;

use crate::api::RouterState;

// Validate confirmation here, just in case.
// It's really easy to send an empty request, these
// required parameters make it harder to accidentally
// delete valuable data.

/// Confirmation info for class deletion
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(in crate::api) struct DeleteAttrConfirmation {
	/// The dataset to delete from.
	/// Should match request.
	pub dataset_name: String,

	/// The class to delete from.
	/// Should match request.
	pub class_name: String,

	/// The attribute to delete
	/// Should match request.
	pub attr_name: String,
}

/// Delete an attribute in the given class
#[utoipa::path(
	delete,
	path = "/{dataset_name}/classes/{class_name}/attrs/{attr_name}",
	tag = "Itemclass",
	params(
		("dataset_name" = String, description = "Dataset name"),
		("class_name" = String, description = "Class name"),
		("attr_name" = String, description = "New attribute name"),
	),
	responses(
		(status = 200, description = "Successfully deleted this attribute"),
		(status = 400, description = "Invalid request", body=String),
		(status = 404, description = "Unknown dataset, class, or attribute", body = String),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(in crate::api) async fn del_class_attr(
	Path((dataset_name, class_name, attr_name)): Path<(String, String, String)>,
	State(state): State<RouterState>,
	Json(confirmation): Json<DeleteAttrConfirmation>,
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

	if confirmation.attr_name != attr_name {
		return (
			StatusCode::BAD_REQUEST,
			format!("Confirmation does not match attribute name"),
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
				format!("Could not get dataset by name"),
			)
				.into_response();
		}
	};

	let class_handle: ClassHandle = match dataset.get_class(&class_name) {
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

	let attr_handle = match dataset.get_attr(class_handle, &attr_name) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{class_name}` does not have the attribute `{attr_name}`"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get attribute by name",
				dataset = dataset_name,
				class_name = ?class_name,
				attr_name = ?attr_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not attribute by name: {e}"),
			)
				.into_response();
		}
	};

	let res = dataset.del_attr(attr_handle);

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(e) => {
			error!(
				message = "Could not delete attribute",
				dataset = dataset_name,
				class_handle = ?class_handle,
				class_name = ?class_name,
				attr_name = attr_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete attribute: {e}"),
			)
				.into_response();
		}
	}
}
