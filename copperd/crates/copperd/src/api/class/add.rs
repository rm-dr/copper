use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_ds_core::{api::meta::Metastore, errors::MetastoreError};
use serde::Deserialize;
use tracing::error;
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, ToSchema, Debug)]
pub(in crate::api) struct NewClassRequest {
	pub dataset: String,
	pub new_class_name: String,
}

/// Create a new class
#[utoipa::path(
	post,
	path = "/add",
	responses(
		(status = 200, description = "Successfully created new class"),
		(status = 400, description = "Could not create new class, bad parameters", body = String),
		(status = 404, description = "Bad dataset", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn add_class(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<NewClassRequest>,
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

	let res = dataset.add_class(&payload.new_class_name).await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateClassName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Class name `{x}` already exists"),
			)
				.into_response()
		}
		Err(MetastoreError::BadClassName(x)) => {
			return (StatusCode::BAD_REQUEST, format!("Bad class name: {x}")).into_response()
		}
		Err(e) => {
			error!(
				message = "Could not create new class",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Could not create new class",
			)
				.into_response();
		}
	}
}
