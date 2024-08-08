use crate::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::error;
use ufo_ds_core::{
	api::meta::Metastore,
	data::MetastoreDataStub,
	handles::{AttrHandle, ClassHandle},
};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ClassInfoRequest {
	dataset: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct ClassInfo {
	/// This class' name
	#[schema(value_type = String)]
	name: SmartString<LazyCompact>,

	/// This class' unique handle
	#[schema(value_type = u32)]
	handle: ClassHandle,

	/// This class' attributes
	attrs: Vec<AttrInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct AttrInfo {
	/// This attribute's name
	#[schema(value_type = String)]
	name: SmartString<LazyCompact>,

	/// This attribute's unique handle
	#[schema(value_type = u32)]
	handle: AttrHandle,

	/// This attribute's data type
	data_type: MetastoreDataStub,
}

/// Get this dataset's classes
#[utoipa::path(
	get,
	path = "/list",
	params(
		ClassInfoRequest
	),
	responses(
		(status = 200, description = "Classes", body = Vec<ClassInfo>),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(super) async fn list_classes(
	State(state): State<RouterState>,
	Query(query): Query<ClassInfoRequest>,
) -> Response {
	let dataset = match state.main_db.get_dataset(&query.dataset).await {
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
				message = "Could not get dataset by name",
				dataset = query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name: {e}"),
			)
				.into_response();
		}
	};

	let classes = match dataset.get_all_classes().await {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get classes",
				dataset = ?query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get classes: {e}"),
			)
				.into_response();
		}
	};

	let mut out = Vec::new();
	for class in classes {
		let attrs = match dataset.class_get_attrs(class.handle).await {
			Ok(x) => x,
			Err(e) => {
				error!(
					message = "Could not get class attributes",
					dataset = ?query.dataset,
					class = ?class,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get class attributes: {e}"),
				)
					.into_response();
			}
		};

		out.push(ClassInfo {
			name: class.name,
			handle: class.handle,
			attrs: attrs
				.into_iter()
				.map(|attr| AttrInfo {
					name: attr.name,
					handle: attr.handle,
					data_type: attr.data_type,
				})
				.collect(),
		});
	}

	return Json(out).into_response();
}
