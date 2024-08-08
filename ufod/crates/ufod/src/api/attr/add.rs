use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tracing::{debug, error};
use ufo_ds_core::{
	api::meta::{AttributeOptions, Metastore},
	data::MetastoreDataStub,
	errors::MetastoreError,
	handles::ClassHandle,
};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Debug, Deserialize, ToSchema)]
pub(super) struct NewAttrParams {
	pub dataset: String,

	#[schema(value_type = u32)]
	pub class: ClassHandle,

	pub new_attr_name: String,

	/// The new attribute's data type
	pub data_type: MetastoreDataStub,

	/// Options for this new attribute
	pub options: AttributeOptions,
}

/// Create a new attribute
#[utoipa::path(
	post,
	path = "/add",
	responses(
		(status = 200, description = "Successfully created new attribute"),
		(status = 400, description = "Could not create new attribute, bad parameters", body = String),
		(status = 404, description = "Bad dataset or class", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn add_attr(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<NewAttrParams>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	debug!(
		message = "Making a new attribute",
		payload = ?payload
	);

	if payload.new_attr_name.is_empty() {
		return (StatusCode::BAD_REQUEST, "Attribute name cannot be empty").into_response();
	} else if payload.new_attr_name.trim() == "" {
		return (
			StatusCode::BAD_REQUEST,
			"Attribute name cannot be whitespace",
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
				message = "Could not get dataset",
				dataset = payload.dataset,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get dataset").into_response();
		}
	};

	let class = match dataset.get_class(payload.class).await {
		Ok(x) => x,
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

	let res = dataset
		.add_attr(
			class.handle,
			&payload.new_attr_name,
			payload.data_type,
			payload.options,
		)
		.await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DuplicateAttrName(x)) => {
			return (
				StatusCode::BAD_REQUEST,
				format!("Attribute `{x}` already exists"),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not create new attribute",
				dataset = payload.dataset,
				class = ?payload.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Could not create new attribute",
			)
				.into_response();
		}
	}
}
