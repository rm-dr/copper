use std::collections::HashMap;

use crate::api::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use copper_ds_core::{api::meta::Metastore, errors::MetastoreError};
use utoipa::{IntoParams, ToSchema};

use super::{ItemListData, ItemListItem};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemGetRequest {
	pub dataset: String,
	pub class: u32,
	pub item: u32,
}

/// Get a specific item in this class
#[utoipa::path(
	get,
	path = "/get",
	params(
		ItemGetRequest
	),
	responses(
		(status = 200, description = "Item information", body = ItemListItem),
		(status = 404, description = "Unknown dataset, class, or item", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn get_item(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<ItemGetRequest>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	let dataset = match state.main_db.dataset.get_dataset(&query.dataset).await {
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
				message = "Could not get dataset",
				dataset = query.dataset,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get dataset").into_response();
		}
	};

	// The scope here is necessary, res must be dropped to avoid an error.
	let item = {
		let res = dataset
			.get_item(query.class.into(), query.item.into())
			.await;

		match res {
			Ok(x) => x,

			Err(MetastoreError::BadItemIdx) => {
				return (
					StatusCode::NOT_FOUND,
					format!("Item `{}` does not exist", query.item),
				)
					.into_response()
			}

			Err(MetastoreError::BadClassHandle) => {
				return (
					StatusCode::NOT_FOUND,
					format!("Class `{}` does not exist", query.class),
				)
					.into_response()
			}

			Err(e) => {
				error!(
					message = "Could not get item",
					query = ?query,
					error = ?e
				);
				return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get item").into_response();
			}
		}
	};

	let attrs = {
		let res = dataset.class_get_attrs(query.class.into()).await;
		match res {
			Ok(x) => x,

			Err(MetastoreError::BadClassHandle) => {
				return (
					StatusCode::NOT_FOUND,
					format!("Class `{}` does not exist", query.class),
				)
					.into_response()
			}

			Err(e) => {
				error!(
					message = "Could not get attrs",
					query = ?query,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get attrs: {e}"),
				)
					.into_response();
			}
		}
	};

	let mut itemlistdata = HashMap::new();
	for (attr, val) in attrs.iter().zip(item.attrs.iter()) {
		let d = match ItemListData::from_data(&dataset, attr.clone(), val).await {
			Ok(x) => x,
			Err(r) => return r,
		};
		itemlistdata.insert(attr.handle.into(), d);
	}

	return Json(ItemListItem {
		idx: item.handle.into(),
		attrs: itemlistdata,
	})
	.into_response();
}
