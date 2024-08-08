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

// TODO: recycle bin?

/// Confirmation info for class deletion
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(in crate::api) struct DeleteDatasetConfirmation {
	/// The dataset to delete from.
	/// Should match request.
	pub dataset_name: String,
}

/// Delete a dataset
#[utoipa::path(
	delete,
	path = "/{dataset_name}",
	params(
		("dataset_name" = String, description = "Dataset name")
	),
	responses(
		(status = 200, description = "Dataset deleted successfully"),
		(status = 400, description = "Could not delete dataset", body = NewDatasetError),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn del_dataset(
	State(state): State<RouterState>,
	Path(dataset_name): Path<String>,
	Json(confirmation): Json<DeleteDatasetConfirmation>,
) -> Response {
	if confirmation.dataset_name != dataset_name {
		return (
			StatusCode::BAD_REQUEST,
			format!("Confirmation does not match dataset name"),
		)
			.into_response();
	}

	let res = state.main_db.del_dataset(&dataset_name);

	match res {
		Ok(_) => {}
		Err(e) => {
			error!(
				message = "Could not delete dataset",
				dataset = dataset_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete dataset `{dataset_name}`"),
			)
				.into_response();
		}
	};

	return StatusCode::OK.into_response();
}
