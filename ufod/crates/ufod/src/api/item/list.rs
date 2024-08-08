use std::collections::HashMap;

use crate::api::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::{
	api::{blob::BlobHandle, meta::Metastore},
	data::{HashType, MetastoreData},
	handles::ItemIdx,
};
use ufo_util::mime::MimeType;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemListRequest {
	pub dataset: String,
	pub class: String,

	/// How many items to list per page
	pub page_size: u32,

	/// The index of the first item to return
	pub start_at: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemListItem {
	idx: u32,
	attrs: HashMap<String, ItemListData>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemListResponse {
	items: Vec<ItemListItem>,
	count: usize,
	total: u32,
	start_at: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub(super) enum ItemListData {
	None,
	PositiveInteger { value: u64 },
	Integer { value: i64 },
	Binary { format: MimeType },
	Float { value: f64 },
	Boolean { value: bool },
	Text { value: String },
	Reference { class: String, item: ItemIdx },
	Hash { hash_type: HashType, value: String },
	Blob { handle: BlobHandle },
}

/// Create a new attribute
#[utoipa::path(
	get,
	path = "/list",
	params(
		ItemListRequest
	),
	responses(
		(status = 200, description = "Items", body=ItemListResponse),
		(status = 400, description = "Could not list items bad parameters", body=String),
		(status = 404, description = "Unknown dataset or class name", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn list_item(
	State(state): State<RouterState>,
	Query(query): Query<ItemListRequest>,
) -> Response {
	// TODO: configure max page size
	if query.page_size > 100 {
		return (
			StatusCode::BAD_REQUEST,
			format!(
				"Page size `{}` exceeds server limit of `{}`",
				query.page_size, 100
			),
		)
			.into_response();
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
				message = "Could not get dataset by name",
				dataset = query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset by name"),
			)
				.into_response();
		}
	};

	let class = match dataset.get_class_by_name(&query.class).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{}` does not exist", query.class),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class by name",
				dataset = query.dataset,
				class_name = ?query.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class by name {e}"),
			)
				.into_response();
		}
	};

	// The scope here is necessary, res must be dropped to avoid an error.
	let itemdata = {
		let res = dataset
			.get_items(class.handle, query.page_size, query.start_at)
			.await;

		match res {
			Ok(x) => x,
			Err(e) => {
				error!(
					message = "Could not get items",
					query = ?query,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get items: {e}"),
				)
					.into_response();
			}
		}
	};

	let attrs = {
		let res = dataset.class_get_attrs(class.handle).await;
		match res {
			Ok(x) => x,
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

	let mut out = Vec::new();
	for r in itemdata.into_iter() {
		let mut itemlistdata = HashMap::new();
		for (attr, val) in attrs.iter().zip(r.attrs.iter()) {
			// TODO: move to method (after making generic dataset type)
			let d = match val {
				MetastoreData::None(_) => ItemListData::None,
				MetastoreData::PositiveInteger(x) => ItemListData::PositiveInteger { value: *x },
				MetastoreData::Blob { handle } => ItemListData::Blob { handle: *handle },
				MetastoreData::Integer(x) => ItemListData::Integer { value: *x },
				MetastoreData::Boolean(x) => ItemListData::Boolean { value: *x },
				MetastoreData::Float(x) => ItemListData::Float { value: *x },
				MetastoreData::Binary { format, .. } => ItemListData::Binary {
					format: format.clone(),
				},
				MetastoreData::Text(t) => ItemListData::Text {
					value: t.to_string(),
				},
				MetastoreData::Hash { format, data } => ItemListData::Hash {
					hash_type: *format,
					value: data.iter().map(|x| format!("{:X?}", x)).join(""),
				},
				MetastoreData::Reference { class, item } => {
					// TODO: make this async
					let class = match dataset.get_class(*class).await {
						Ok(x) => x,
						Err(e) => {
							error!(
								message = "Could not get class by name",
								dataset = query.dataset,
								class_name = ?query.class,
								error = ?e
							);
							return (
								StatusCode::INTERNAL_SERVER_ERROR,
								format!("Could not get class by name {e}"),
							)
								.into_response();
						}
					};

					ItemListData::Reference {
						class: class.name.into(),
						item: *item,
					}
				}
			};

			itemlistdata.insert(attr.name.to_string(), d);
		}

		out.push(ItemListItem {
			idx: r.handle.into(),
			attrs: itemlistdata,
		})
	}

	return Json(ItemListResponse {
		count: out.len(),
		start_at: query.start_at,
		total: 0, // TODO: return total item count
		items: out,
	})
	.into_response();
}
