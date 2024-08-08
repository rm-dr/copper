use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::error;
use ufo_ds_core::{
	data::MetastoreDataStub,
	handles::{AttrHandle, ClassHandle},
};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(in crate::api) struct ItemclassInfo {
	/// This item class' name
	#[schema(value_type = String)]
	name: SmartString<LazyCompact>,

	/// This item class' unique handle
	#[schema(value_type = u32)]
	handle: ClassHandle,

	/// This item class' attributes
	attrs: Vec<AttrInfo>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(in crate::api) struct AttrInfo {
	/// This attribute's name
	#[schema(value_type = String)]
	name: SmartString<LazyCompact>,

	/// This attribute's unique handle
	#[schema(value_type = u32)]
	handle: AttrHandle,

	/// This attribute's data type
	data_type: MetastoreDataStub,
}

/// Get this dataset's itemclasses
#[utoipa::path(
	get,
	path = "/{dataset_name}/classes",
	tag = "Itemclass",
	params(
		("dataset_name" = String, description = "Dataset name"),
	),
	responses(
		(status = 200, description = "Item classes", body = Vec<ItemclassInfo>),
		(status = 500, description = "Internal server error", body = String),
	),
)]
pub(in crate::api) async fn list_itemclasses(
	Path(dataset_name): Path<String>,
	State(state): State<RouterState>,
) -> Response {
	let dataset = state.main_db.get_dataset(&dataset_name).unwrap().unwrap();

	let classes = match dataset.get_all_classes() {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get item classes",
				dataset = ?dataset_name,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get classes: {e:?}"),
			)
				.into_response();
		}
	};

	let mut out = Vec::new();
	for (class_handle, class_name) in classes {
		let attrs = match dataset.class_get_attrs(class_handle) {
			Ok(x) => x,
			Err(e) => {
				error!(
					message = "Could not get item class attributes",
					dataset = ?dataset_name,
					item_class_handle = ?class_handle,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get attributes: {e:?}"),
				)
					.into_response();
			}
		};

		out.push(ItemclassInfo {
			name: class_name,
			handle: class_handle.into(),
			attrs: attrs
				.into_iter()
				.map(|(handle, name, data_type)| AttrInfo {
					name,
					handle: handle.into(),
					data_type,
				})
				.collect(),
		});
	}

	return Json(out).into_response();
}
