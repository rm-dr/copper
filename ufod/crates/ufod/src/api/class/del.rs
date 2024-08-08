use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use tracing::error;

use super::ClassSelect;
use crate::api::RouterState;

/// Delete a class and all data associated with it
#[utoipa::path(
	delete,
	path = "/del",
	responses(
		(status = 200, description = "Successfully deleted this class"),
		(status = 400, description = "Invalid request", body=String),
		(status = 404, description = "Invalid dataset or class", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn del_class(
	State(state): State<RouterState>,
	Json(payload): Json<ClassSelect>,
) -> Response {
	let dataset = match state.main_db.get_dataset(&payload.dataset) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", payload.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name: {e}"),
			)
				.into_response();
		}
	};

	let class_handle = match dataset.get_class(&payload.class) {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{}` does not exist", payload.class),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class by name",
				dataset = payload.dataset,
				class_name = ?payload.class,
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
				dataset = payload.dataset,
				class_name = payload.class,
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
