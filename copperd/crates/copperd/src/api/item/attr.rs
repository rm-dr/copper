use crate::api::RouterState;
use axum::{
	body::Body,
	extract::{Query, State},
	http::{header, StatusCode},
	response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use copper_ds_core::{
	api::{blob::Blobstore, meta::Metastore},
	data::MetastoreData,
	errors::MetastoreError,
};
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use tracing::error;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ItemAttrRequest {
	pub dataset: String,

	pub attr: u32,
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
		(status = 400, description = "Could not get this attribute", body = String),
		(status = 404, description = "Invalid dataset, class, or item", body = String),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn get_item_attr(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<ItemAttrRequest>,
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
				message = "Could not get dataset by name",
				dataset = query.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				"Could not get dataset by name",
			)
				.into_response();
		}
	};

	let attr = match dataset.get_attr(query.attr.into()).await {
		Ok(x) => x,
		Err(MetastoreError::BadAttrHandle) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Attr `{:?}` does not exist", query.attr),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get attr",
				dataset = query.dataset,
				attr = ?query.attr,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get attr").into_response();
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
				attr = ?query.attr,
				item_idx = ?query.item_idx,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get item").into_response();
		}
	};

	return match attr_value {
		MetastoreData::None(_) => StatusCode::OK.into_response(),
		MetastoreData::Text(t) => t.to_string().into_response(),
		MetastoreData::Integer { value, .. } => format!("{value}").into_response(),
		MetastoreData::Float { value, .. } => format!("{value}").into_response(),
		MetastoreData::Boolean(x) => format!("{x}").into_response(),
		MetastoreData::Hash { data, .. } => MetastoreData::hash_to_string(&data).into_response(),
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
						attr = ?query.attr,
						item_idx = ?query.item_idx,
						error = ?e
					);
					return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get blob")
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
