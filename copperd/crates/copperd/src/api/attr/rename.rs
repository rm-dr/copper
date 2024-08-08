use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tracing::error;
use copper_ds_core::{api::meta::Metastore, errors::MetastoreError, handles::AttrHandle};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, ToSchema, Debug)]
pub(in crate::api) struct RenameAttrRequest {
	pub dataset: String,

	#[schema(value_type = u32)]
	pub attr: AttrHandle,

	pub new_name: String,
}

/// Create a new class
#[utoipa::path(
	post,
	path = "/rename",
	responses(
		(status = 200, description = "Successfully renamed this attr"),
		(status = 400, description = "Could not rename attr, bad parameters", body = String),
		(status = 404, description = "Bad dataset", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn rename_attr(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<RenameAttrRequest>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	// TODO: ONE function to check name
	if payload.new_name.is_empty() {
		return (StatusCode::BAD_REQUEST, "Class name cannot be empty").into_response();
	} else if payload.new_name.trim() == "" {
		return (StatusCode::BAD_REQUEST, "Class name cannot be whitespace").into_response();
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

	let res = dataset.attr_set_name(payload.attr, &payload.new_name).await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateAttrName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("An attribute named `{x}` already exists"),
			)
				.into_response()
		}
		Err(MetastoreError::BadAttrHandle) => {
			return (StatusCode::BAD_REQUEST, format!("Invalid attribute handle")).into_response()
		}
		Err(MetastoreError::BadAttrName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Invalid attribute name: {x}"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not rename attribute",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Could not rename attribute",
			)
				.into_response();
		}
	}
}
