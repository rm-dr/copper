use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_ds_core::{api::meta::Metastore, errors::MetastoreError, handles::ClassHandle};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, ToSchema, Debug)]
pub(in crate::api) struct RenameClassRequest {
	pub dataset: String,

	#[schema(value_type = u32)]
	pub class: ClassHandle,

	pub new_name: String,
}

/// Create a new class
#[utoipa::path(
	post,
	path = "/rename",
	responses(
		(status = 200, description = "Successfully renamed this class"),
		(status = 400, description = "Could not rename class, bad parameters", body = String),
		(status = 404, description = "Bad dataset", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn rename_class(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<RenameClassRequest>,
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

	let res = dataset
		.class_set_name(payload.class, &payload.new_name)
		.await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateClassName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("A class named `{x}` already exists"),
			)
				.into_response()
		}
		Err(MetastoreError::BadClassHandle) => {
			return (StatusCode::BAD_REQUEST, "Invalid class handle").into_response()
		}
		Err(MetastoreError::BadClassName(x)) => {
			return (StatusCode::BAD_REQUEST, format!("Invalid class name: {x}")).into_response()
		}
		Err(e) => {
			error!(
				message = "Could not rename class",
				dataset = payload.dataset,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not rename class").into_response();
		}
	}
}
