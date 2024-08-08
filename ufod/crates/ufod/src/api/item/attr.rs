use crate::api::RouterState;
use axum::{
	body::Body,
	extract::{Query, State},
	http::{header, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use tracing::error;
use ufo_ds_core::{api::blob::Blobstore, api::meta::Metastore, data::MetastoreData};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemAttrRequest {
	pub dataset: String,
	pub class: String,
	pub attr: String,
	pub item_idx: u32,
}

/// Get an item's attribute value as raw data
#[utoipa::path(
	get,
	path = "/attr",
	params(
		ItemAttrRequest
	),
	responses(
		(status = 200, description = "Item data"),
		(status = 400, description = "Could not get this attribute", body=String),
		(status = 404, description = "Invalid dataset, class, or item", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn get_item_attr(
	State(state): State<RouterState>,
	Query(query): Query<ItemAttrRequest>,
) -> Response {
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
				format!("Could not get class by name"),
			)
				.into_response();
		}
	};

	let attr = match dataset.get_attr_by_name(class.handle, &query.attr).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Attr `{}` does not exist", query.attr),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get attr by name",
				dataset = query.dataset,
				class_name = ?query.class,
				attr_name = ?query.attr,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get attr by name"),
			)
				.into_response();
		}
	};

	let attr_value = match dataset
		.get_item_attr(attr.handle, query.item_idx.into())
		.await
	{
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get item",
				dataset = query.dataset,
				class_name = ?query.class,
				attr_name = ?query.attr,
				item_idx = ?query.item_idx,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get item"),
			)
				.into_response();
		}
	};

	return match attr_value {
		MetastoreData::None(_) => StatusCode::OK.into_response(),
		MetastoreData::Text(t) => t.to_string().into_response(),
		MetastoreData::Integer(x) => format!("{x}").into_response(),
		MetastoreData::PositiveInteger(x) => format!("{x}").into_response(),
		MetastoreData::Boolean(x) => format!("{x}").into_response(),
		MetastoreData::Float(x) => format!("{x}").into_response(),
		MetastoreData::Hash { data, .. } => data
			.iter()
			.map(|x| format!("{:X?}", x))
			.join("")
			.into_response(),
		MetastoreData::Binary { mime, data } => {
			let body = Body::from((*data).clone());
			let headers = AppendHeaders([(header::CONTENT_TYPE, mime.to_string())]);
			(headers, body).into_response()
		}
		MetastoreData::Blob { handle } => {
			let blob = match dataset.get_blob(handle).await {
				Ok(x) => x,
				Err(e) => {
					error!(
						message = "Could not get blob",
						dataset = query.dataset,
						class_name = ?query.class,
						attr_name = ?query.attr,
						item_idx = ?query.item_idx,
						error = ?e
					);
					return (
						StatusCode::INTERNAL_SERVER_ERROR,
						format!("Could not get blob"),
					)
						.into_response();
				}
			};

			let body = Body::from_stream(ReaderStream::new(blob.data));
			let headers = AppendHeaders([(header::CONTENT_TYPE, blob.mime.to_string())]);
			(headers, body).into_response()
		}

		// References may not be viewed as raw data
		MetastoreData::Reference { .. } => StatusCode::BAD_REQUEST.into_response(),
	};
}
