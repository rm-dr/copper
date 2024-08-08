use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tracing::error;
use ufo_ds_core::{api::meta::Metastore, errors::MetastoreError};
use utoipa::{IntoParams, ToSchema};

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub(super) struct GetAttrRequest {
	pub dataset: String,
	pub attr: u32,
}

/// Find an attribute by name
#[utoipa::path(
	get,
	path = "/get",
	params(
		GetAttrRequest
	),
	responses(
		(status = 200, description = "Attribute info", body = AttrInfo),
		(status = 404, description = "Could not find attribute", body = String),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(super) async fn get_attr(
	jar: CookieJar,
	Query(payload): Query<GetAttrRequest>,
	State(state): State<RouterState>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let dataset = match state.main_db.dataset.get_dataset(&payload.dataset).await {
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
				message = "Could not get dataset",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset"),
			)
				.into_response();
		}
	};

	match dataset.get_attr(payload.attr.into()).await {
		Ok(x) => return Json(x).into_response(),
		Err(MetastoreError::BadAttrHandle) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Attr `{:?}` does not exist", payload.attr),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get attr",
				dataset = payload.dataset,
				attr = ?payload.attr,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get attr"),
			)
				.into_response();
		}
	};
}
