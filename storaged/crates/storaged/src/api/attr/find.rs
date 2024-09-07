use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_ds_core::{api::meta::Metastore, errors::MetastoreError};
use serde::Deserialize;
use tracing::error;
use utoipa::{IntoParams, ToSchema};

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub(super) struct FindAttrRequest {
	pub dataset: String,
	// #[schema(value_type = u32)] doesn't seem to work here,
	// because of `IntoParams`
	pub class: u32,
	pub attr_name: String,
}

/// Find an attribute by name
#[utoipa::path(
	get,
	path = "/find",
	params(
		FindAttrRequest
	),
	responses(
		(status = 200, description = "Attribute info", body = AttrInfo),
		(status = 404, description = "Could not find attribute", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn find_attr(
	jar: CookieJar,
	Query(payload): Query<FindAttrRequest>,
	State(state): State<RouterState>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
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
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get dataset").into_response();
		}
	};

	match dataset
		.get_attr_by_name(payload.class.into(), &payload.attr_name)
		.await
	{
		Ok(Some(x)) => return Json(x).into_response(),
		Ok(None) => return (StatusCode::NOT_FOUND, "No such attribute").into_response(),
		Err(MetastoreError::BadClassHandle) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{:?}` does not exist", payload.class),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class",
				dataset = payload.dataset,
				class = ?payload.class,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get class").into_response();
		}
	};
}
