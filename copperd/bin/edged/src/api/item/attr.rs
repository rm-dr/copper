use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::{body::AsyncReadBody, extract::CookieJar};
use copper_itemdb::{
	client::base::{
		client::ItemdbClient,
		errors::{class::GetClassError, dataset::GetDatasetError, item::GetItemError},
	},
	AttrData, AttributeId,
};
use tracing::error;

/// Get the value of an item's attribute
#[utoipa::path(
	get,
	path = "/{item_idx}/attr/{attr_idx}",
	params(
		("item_idx", description = "Item id"),
		("attr_idx", description = "Attribute id"),
	),
	responses(
		(status = 200, description = "The attribute's data", body = Vec<DatasetInfo>),
		(status = 500, description = "Internal server error"),
	),
)]
pub(super) async fn get_attr<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path((item_id, attr_id)): Path<(i64, i64)>,
) -> Response {
	let attr_id: AttributeId = attr_id.into();
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let item = match state.itemdb_client.get_item(item_id.into()).await {
		Ok(x) => x,

		Err(GetItemError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetItemError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	// TODO: do permission checks in one query
	let class = match state.itemdb_client.get_class(item.class).await {
		Ok(x) => x,

		Err(GetClassError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	match state.itemdb_client.get_dataset(class.dataset).await {
		Ok(x) => {
			if x.owner != user.id {
				return StatusCode::UNAUTHORIZED.into_response();
			}
		}

		Err(GetDatasetError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetDatasetError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	// Find and return attribute
	if let Some(value) = item.attribute_values.get(&attr_id) {
		match value {
			AttrData::Blob { bucket, key } => {
				let stream = match state.s3_client.get_object_stream(bucket, key).await {
					Ok(x) => x,
					Err(error) => {
						error!(message = "Error in s3 client", ?error);
						return StatusCode::INTERNAL_SERVER_ERROR.into_response();
					}
				};

				return (StatusCode::OK, AsyncReadBody::new(stream.into_async_read()))
					.into_response();
			}

			_ => {
				return (
					StatusCode::BAD_REQUEST,
					Json(format!(
						"attributes of type {:?} cannot be serialized",
						value.as_stub()
					)),
				)
					.into_response();
			}
		}
	} else {
		return StatusCode::NOT_FOUND.into_response();
	}
}
