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
use ufo_ds_core::{
	api::{
		blob::{BlobHandle, Blobstore},
		meta::{AttrInfo, Metastore},
	},
	data::{HashType, MetastoreData, MetastoreDataStub},
	errors::MetastoreError,
	handles::{ClassHandle, ItemIdx},
};
use ufo_ds_impl::local::LocalDataset;
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
	pub idx: u32,
	pub attrs: HashMap<u32, ItemListData>,
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
	Integer {
		attr: AttrInfo,
		is_non_negative: bool,
		value: Option<i64>,
	},
	Float {
		attr: AttrInfo,
		is_non_negative: bool,
		value: Option<f64>,
	},
	Boolean {
		attr: AttrInfo,
		value: Option<bool>,
	},
	Text {
		attr: AttrInfo,
		value: Option<String>,
	},
	Reference {
		attr: AttrInfo,
		#[schema(value_type = u32)]
		class: ClassHandle,

		#[schema(value_type = Option<u32>)]
		item: Option<ItemIdx>,
	},
	Hash {
		attr: AttrInfo,
		hash_type: HashType,
		value: Option<String>,
	},
	Binary {
		attr: AttrInfo,
		#[schema(value_type = Option<String>)]
		mime: Option<MimeType>,
		size: Option<u64>,
	},
	Blob {
		attr: AttrInfo,
		#[schema(value_type = Option<String>)]
		mime: Option<MimeType>,
		handle: Option<BlobHandle>,
		size: Option<u64>,
	},
}

impl ItemListData {
	pub async fn from_data(
		dataset: &LocalDataset,
		attr: AttrInfo,
		data: &MetastoreData,
	) -> Result<Self, Response> {
		Ok(match data {
			MetastoreData::None(t) => match t {
				// These must match the serialized tags of `ItemListData`
				MetastoreDataStub::Text => ItemListData::Text {
					value: None,
					attr: attr.clone(),
				},
				MetastoreDataStub::Binary => ItemListData::Binary {
					attr: attr.clone(),
					mime: None,
					size: None,
				},
				MetastoreDataStub::Blob => ItemListData::Blob {
					attr: attr.clone(),
					mime: None,
					handle: None,
					size: None,
				},
				MetastoreDataStub::Integer { is_non_negative } => ItemListData::Integer {
					attr: attr.clone(),
					is_non_negative: *is_non_negative,
					value: None,
				},
				MetastoreDataStub::Boolean => ItemListData::Boolean {
					attr: attr.clone(),
					value: None,
				},
				MetastoreDataStub::Float { is_non_negative } => ItemListData::Float {
					attr: attr.clone(),
					is_non_negative: *is_non_negative,
					value: None,
				},
				MetastoreDataStub::Hash { hash_type } => ItemListData::Hash {
					attr: attr.clone(),
					hash_type: *hash_type,
					value: None,
				},
				MetastoreDataStub::Reference { class } => ItemListData::Reference {
					attr: attr.clone(),
					class: *class,
					item: None,
				},
			},
			MetastoreData::Blob { handle } => {
				let size = match dataset.blob_size(*handle).await {
					Ok(x) => x,
					Err(e) => {
						error!(
							message = "Could not get blob length",
							blob = ?handle,
							error = ?e
						);
						return Err((
							StatusCode::INTERNAL_SERVER_ERROR,
							"Could not get blob length",
						)
							.into_response());
					}
				};

				ItemListData::Blob {
					attr: attr.clone(),
					mime: Some(MimeType::Flac),
					handle: Some(*handle),
					size: Some(size),
				}
			}
			MetastoreData::Integer {
				value,
				is_non_negative,
			} => ItemListData::Integer {
				is_non_negative: *is_non_negative,
				attr: attr.clone(),
				value: Some(*value),
			},
			MetastoreData::Boolean(x) => ItemListData::Boolean {
				attr: attr.clone(),
				value: Some(*x),
			},
			MetastoreData::Float {
				value,
				is_non_negative,
			} => ItemListData::Float {
				is_non_negative: *is_non_negative,
				attr: attr.clone(),
				value: Some(*value),
			},
			MetastoreData::Binary { mime, data } => ItemListData::Binary {
				attr: attr.clone(),
				mime: Some(mime.clone()),
				size: Some(data.len().try_into().unwrap()),
			},
			MetastoreData::Text(t) => ItemListData::Text {
				attr: attr.clone(),
				value: Some(t.to_string()),
			},
			MetastoreData::Hash { format, data } => ItemListData::Hash {
				attr: attr.clone(),
				hash_type: *format,
				value: Some(MetastoreData::hash_to_string(data)),
			},
			MetastoreData::Reference { class, item } => ItemListData::Reference {
				attr: attr.clone(),
				class: *class,
				item: Some(*item),
			},
		})
	}
}

/// List all items in a class
#[utoipa::path(
	get,
	path = "/list",
	params(
		ItemListRequest
	),
	responses(
		(status = 200, description = "Items", body = ItemListResponse),
		(status = 400, description = "Could not list items bad parameters", body = String),
		(status = 404, description = "Unknown dataset or class name", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn list_item(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<ItemListRequest>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
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
				message = "Could not get dataset",
				dataset = query.dataset,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get dataset").into_response();
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
				return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get items").into_response();
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
			let d = match ItemListData::from_data(&dataset, attr.clone(), val).await {
				Ok(x) => x,
				Err(r) => return r,
			};
			itemlistdata.insert(attr.handle.into(), d);
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
