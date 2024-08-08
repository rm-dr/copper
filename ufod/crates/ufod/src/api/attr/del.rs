use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use tracing::error;
use ufo_ds_core::api::meta::Metastore;

use super::AttrSelect;
use crate::api::RouterState;

/// Delete an attribute
#[utoipa::path(
	delete,
	path = "/del",
	responses(
		(status = 200, description = "Successfully deleted this attribute"),
		(status = 400, description = "Invalid request", body = String),
		(status = 404, description = "Unknown dataset, class, or attribute", body = String),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(super) async fn del_attr(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<AttrSelect>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let dataset = match state
		.main_db
		.dataset
		.get_dataset(&payload.class.dataset)
		.await
	{
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", payload.class.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = payload.class.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name"),
			)
				.into_response();
		}
	};

	let class = match dataset.get_class_by_name(&payload.class.class).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{}` does not exist", payload.class.class),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class by name",
				dataset = payload.class.dataset,
				payload.class.class = ?payload.class.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class by name: {e}"),
			)
				.into_response();
		}
	};

	let attr = match dataset.get_attr_by_name(class.handle, &payload.attr).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!(
					"Class `{}` does not have the attribute `{}`",
					payload.class.class, payload.attr
				),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get attribute by name",
				dataset = payload.class.dataset,
				payload.class.class = ?payload.class.class,
				attr_name = ?payload.attr,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not attribute by name: {e}"),
			)
				.into_response();
		}
	};

	let res = dataset.del_attr(attr.handle).await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(e) => {
			error!(
				message = "Could not delete attribute",
				dataset = payload.class.dataset,
				class = ?class,
				payload.class.class = ?payload.class.class,
				attr_name = payload.attr,
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
