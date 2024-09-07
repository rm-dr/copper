use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_ds_core::{api::meta::Metastore, errors::MetastoreError, handles::AttrHandle};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct DelAttrRequest {
	pub dataset: String,

	#[schema(value_type = u32)]
	pub attr: AttrHandle,
}

/// Delete an attribute
#[utoipa::path(
	delete,
	path = "/del",
	responses(
		(status = 200, description = "Successfully deleted attribute"),
		(status = 400, description = "Invalid request", body = String),
		(status = 404, description = "Bad dataset, class, or attribute", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn del_attr(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<DelAttrRequest>,
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

	let attr = match dataset.get_attr(payload.attr).await {
		Ok(x) => x,
		Err(MetastoreError::BadAttrHandle) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Attribute `{:?}` does not exist", payload.attr),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get attribute",
				dataset = payload.dataset,
				attr = ?payload.attr,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get attribute").into_response();
		}
	};

	let res = dataset.del_attr(attr.handle).await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(e) => {
			error!(
				message = "Could not delete attribute",
				dataset = payload.dataset,
				attr = ?payload.attr,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Could not delete attribute",
			)
				.into_response();
		}
	}
}
