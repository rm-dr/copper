use std::collections::HashMap;

use crate::api::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore},
		meta::Metastore,
	},
	data::{HashType, MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::{AttrHandle, ItemIdx},
};
use ufo_util::mime::MimeType;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemListRequest {
	pub dataset: String,
	pub class: u32,

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
	PositiveInteger {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		value: Option<u64>,
	},
	Integer {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		value: Option<i64>,
	},
	Float {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		value: Option<f64>,
	},
	Boolean {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		value: Option<bool>,
	},
	Text {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		value: Option<String>,
	},
	Reference {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		class: String,
		item: Option<ItemIdx>,
	},
	Hash {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		hash_type: HashType,
		value: Option<String>,
	},
	Binary {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		#[schema(value_type = Option<String>)]
		mime: Option<MimeType>,
		size: Option<u64>,
	},
	Blob {
		#[schema(value_type = u32)]
		attr: AttrHandle,
		#[schema(value_type = Option<String>)]
		mime: Option<MimeType>,
		handle: Option<BlobHandle>,
		size: Option<u64>,
	},
}

/// List all items in a class
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
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<ItemListRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

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
	// The scope here is necessary, res must be dropped to avoid an error.
	let itemdata = {
		let res = dataset
			.get_items(query.class.into(), query.page_size, query.start_at)
			.await;

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
					message = "Could not get items",
					query = ?query,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get items"),
				)
					.into_response();
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

	let mut out = Vec::new();
	for item in itemdata.into_iter() {
		let mut itemlistdata = HashMap::new();
		for (attr, val) in attrs.iter().zip(item.attrs.iter()) {
			// TODO: move to method (after making generic dataset type)
			let d = match val {
				MetastoreData::None(t) => match t {
					// These must match the serialized tags of `ItemListData`
					MetastoreDataStub::Text => ItemListData::Text {
						value: None,
						attr: attr.handle,
					},
					MetastoreDataStub::Binary => ItemListData::Binary {
						attr: attr.handle,
						mime: None,
						size: None,
					},
					MetastoreDataStub::Blob => ItemListData::Blob {
						attr: attr.handle,
						mime: None,
						handle: None,
						size: None,
					},
					MetastoreDataStub::Integer => ItemListData::Integer {
						attr: attr.handle,
						value: None,
					},
					MetastoreDataStub::PositiveInteger => ItemListData::PositiveInteger {
						attr: attr.handle,
						value: None,
					},
					MetastoreDataStub::Boolean => ItemListData::Boolean {
						attr: attr.handle,
						value: None,
					},
					MetastoreDataStub::Float => ItemListData::Float {
						attr: attr.handle,
						value: None,
					},
					MetastoreDataStub::Hash { hash_type } => ItemListData::Hash {
						attr: attr.handle,
						hash_type: *hash_type,
						value: None,
					},
					MetastoreDataStub::Reference { class } => {
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
									format!("Could not get class by name"),
								)
									.into_response();
							}
						};

						ItemListData::Reference {
							attr: attr.handle,
							class: class.name.to_string(),
							item: None,
						}
					}
				},
				MetastoreData::PositiveInteger(x) => ItemListData::PositiveInteger {
					attr: attr.handle,
					value: Some(*x),
				},
				MetastoreData::Blob { handle } => {
					let size = match dataset.blob_size(*handle).await {
						Ok(x) => x,
						Err(e) => {
							error!(
								message = "Could not get blob length",
								dataset = query.dataset,
								blob = ?handle,
								error = ?e
							);
							return (
								StatusCode::INTERNAL_SERVER_ERROR,
								format!("Could not get blob length"),
							)
								.into_response();
						}
					};

					ItemListData::Blob {
						attr: attr.handle,
						mime: Some(MimeType::Flac),
						handle: Some(*handle),
						size: Some(size),
					}
				}
				MetastoreData::Integer(x) => ItemListData::Integer {
					attr: attr.handle,
					value: Some(*x),
				},
				MetastoreData::Boolean(x) => ItemListData::Boolean {
					attr: attr.handle,
					value: Some(*x),
				},
				MetastoreData::Float(x) => ItemListData::Float {
					attr: attr.handle,
					value: Some(*x),
				},
				MetastoreData::Binary { mime, data } => ItemListData::Binary {
					attr: attr.handle,
					mime: Some(mime.clone()),
					size: Some(data.len().try_into().unwrap()),
				},
				MetastoreData::Text(t) => ItemListData::Text {
					attr: attr.handle,
					value: Some(t.to_string()),
				},
				MetastoreData::Hash { format, data } => ItemListData::Hash {
					attr: attr.handle,
					hash_type: *format,
					value: Some(data.iter().map(|x| format!("{:X?}", x)).join("")),
				},
				MetastoreData::Reference { class, item } => {
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
								format!("Could not get class by name"),
							)
								.into_response();
						}
					};

					ItemListData::Reference {
						attr: attr.handle,
						class: class.name.into(),
						item: Some(*item),
					}
				}
			};

			itemlistdata.insert(attr.name.to_string(), d);
		}

		out.push(ItemListItem {
			idx: item.handle.into(),
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
