use crate::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::api::meta::Metastore;
use utoipa::{IntoParams, ToSchema};

use super::ExtendedClassInfo;

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ClassGetRequest {
	dataset: String,
	class: u32,
}

/// Get class info by id
#[utoipa::path(
	get,
	path = "/get",
	params(
		ClassGetRequest
	),
	responses(
		(status = 200, description = "Class info", body = ExtendedClassInfo),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(super) async fn get_class(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<ClassGetRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let dataset = match state.main_db.dataset.get_dataset(&query.dataset).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", query.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset",
				dataset = query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset"),
			)
				.into_response();
		}
	};

	let class = match dataset.get_class(query.class.into()).await {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get class",
				dataset = ?query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class"),
			)
				.into_response();
		}
	};

	let attrs = match dataset.class_get_attrs(query.class.into()).await {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get class attributes",
				dataset = ?query.dataset,
				class = ?query.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class attributes: {e}"),
			)
				.into_response();
		}
	};

	return Json(ExtendedClassInfo {
		name: class.name,
		handle: class.handle,
		attrs,
	})
	.into_response();
}
