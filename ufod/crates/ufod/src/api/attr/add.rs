use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use ufo_ds_core::{
	api::meta::AttributeOptions, api::meta::Metastore, data::MetastoreDataStub,
	errors::MetastoreError, handles::ClassHandle,
};
use utoipa::ToSchema;

use super::AttrSelect;
use crate::api::RouterState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(in crate::api) struct NewClassAttrParams {
	#[serde(flatten)]
	pub attr: AttrSelect,

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
		(status = 400, description = "Could not create new attribute, bad parameters", body=String),
		(status = 404, description = "Unknown dataset or class name", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(in crate::api) async fn add_attr(
	State(state): State<RouterState>,
	Json(payload): Json<NewClassAttrParams>,
) -> Response {
	debug!(
		message = "Making a new attribute",
		payload = ?payload
	);

	if payload.attr.attr == "" {
		return (
			StatusCode::BAD_REQUEST,
			format!("Attribute name cannot be empty"),
		)
			.into_response();
	} else if payload.attr.attr.trim() == "" {
		return (
			StatusCode::BAD_REQUEST,
			format!("Attribute name cannot be whitespace"),
		)
			.into_response();
	}

	let dataset = match state.main_db.get_dataset(&payload.attr.class.dataset).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", payload.attr.class.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset by name",
				dataset = payload.attr.class.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name"),
			)
				.into_response();
		}
	};

	let class_handle: ClassHandle = match dataset.get_class(&payload.attr.class.class).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{}` does not exist", payload.attr.class.class),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class by name",
				dataset = payload.attr.class.dataset,
				class_name = ?payload.attr.class.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class by name {e}"),
			)
				.into_response();
		}
	};

	let res = dataset
		.add_attr(
			class_handle,
			&payload.attr.attr,
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
				dataset = payload.attr.class.dataset,
				class = ?payload.attr.class.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not create new attribute: {e}"),
			)
				.into_response();
		}
	}
}
