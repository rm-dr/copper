use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use tracing::error;
use ufo_ds_core::{api::meta::Metastore, errors::MetastoreError};

use super::ClassSelect;
use crate::api::RouterState;

/// Create a new class
#[utoipa::path(
	post,
	path = "/add",
	responses(
		(status = 200, description = "Successfully created new class"),
		(status = 400, description = "Could not create new class, bad parameters", body=String),
		(status = 404, description = "This dataset doesn't exist", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn add_class(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<ClassSelect>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	// TODO: ONE function to check name?
	if payload.class == "" {
		return (
			StatusCode::BAD_REQUEST,
			format!("Class name cannot be empty"),
		)
			.into_response();
	} else if payload.class.trim() == "" {
		return (
			StatusCode::BAD_REQUEST,
			format!("Class name cannot be whitespace"),
		)
			.into_response();
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
				message = "Could not get dataset by name",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name"),
			)
				.into_response();
		}
	};

	let res = dataset.add_class(&payload.class).await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateClassName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Class name `{x}` already exists"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not create new class",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not create new class: {e}"),
			)
				.into_response();
		}
	}
}
