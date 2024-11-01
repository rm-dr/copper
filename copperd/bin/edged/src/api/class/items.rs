use std::collections::BTreeMap;

use crate::database::base::client::DatabaseClient;
use crate::RouterState;
use axum::{
	extract::{Path, Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::{
	client::base::{
		client::ItemdbClient,
		errors::{
			class::GetClassError,
			dataset::GetDatasetError,
			item::{CountItemsError, ListItemsError},
		},
	},
	AttrData, AttributeId, ClassId, ItemId,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::{IntoParams, ToSchema};

/// Attribute data returned to the user
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type")]
pub(super) enum ItemAttrData {
	None,

	Text {
		value: String,
	},

	Integer {
		value: i64,
	},

	Float {
		value: f64,
	},

	Boolean {
		value: bool,
	},

	Hash {
		value: String,
	},

	Blob {
		mime: String,
		size: Option<i64>,
	},

	Reference {
		#[schema(value_type = i64)]
		class: ClassId,

		#[schema(value_type = i64)]
		item: ItemId,
	},
}

impl ItemAttrData {
	async fn from_attr_data<Client: DatabaseClient, Itemdb: ItemdbClient>(
		state: &RouterState<Client, Itemdb>,
		value: AttrData,
	) -> Result<Self, Response> {
		Ok(match value {
			AttrData::None { .. } => ItemAttrData::None,
			AttrData::Blob { bucket, key } => {
				let meta = match state
					.s3_client
					.get_object_metadata(bucket.as_str(), key.as_str())
					.await
				{
					Ok(x) => x,
					Err(error) => {
						error!(message = "Error in itemdb client", ?error);
						return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
					}
				};

				ItemAttrData::Blob {
					mime: meta.mime.to_string(),
					size: meta.size,
				}
			}
			AttrData::Integer { value, .. } => ItemAttrData::Integer { value },
			AttrData::Float { value, .. } => ItemAttrData::Float { value },
			AttrData::Boolean { value } => ItemAttrData::Boolean { value },
			AttrData::Reference { class, item } => ItemAttrData::Reference { class, item },

			AttrData::Text { value } => ItemAttrData::Text {
				value: value.into(),
			},

			AttrData::Hash { data, .. } => ItemAttrData::Hash {
				value: data.into_iter().map(|x| format!("{x:02X?}")).join(""),
			},
		})
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ItemlistItemInfo {
	/// The id of this item
	#[schema(value_type = i64)]
	pub id: ItemId,

	/// The class this item belongs to
	#[schema(value_type = i64)]
	pub class: ClassId,

	/// All attributes this item has
	#[schema(value_type = BTreeMap<i64, ItemAttrData>)]
	pub attribute_values: BTreeMap<AttributeId, ItemAttrData>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub(super) struct ItemListResponse {
	skip: i64,
	total: i64,
	items: Vec<ItemlistItemInfo>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub(super) struct PaginateParams {
	skip: i64,
	count: usize,
}

/// Get class info
#[utoipa::path(
	get,
	path = "/{class_id}/items",
	params(
		PaginateParams,
		("class_id", description = "Class id"),
	),
	responses(
		(status = 200, description = "Class info", body = ItemListResponse),
		(status = 401, description = "Unauthorized"),
		(status = 404, description = "Class not found"),
		(status = 500, description = "Internal server error"),
	)
)]
pub(super) async fn list_items<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Query(paginate): Query<PaginateParams>,
	Path(class_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let class = match state.itemdb_client.get_class(class_id.into()).await {
		Ok(x) => x,

		Err(GetClassError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	match state.itemdb_client.get_dataset(class.dataset).await {
		Ok(x) => {
			// We can only modify our own class
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

	let total = match state.itemdb_client.count_items(class.id).await {
		Ok(x) => x,
		Err(CountItemsError::ClassNotFound) => return StatusCode::NOT_FOUND.into_response(),
		Err(CountItemsError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	match state
		.itemdb_client
		.list_items(class.id, paginate.skip, paginate.count)
		.await
	{
		Ok(x) => {
			let mut items = Vec::new();

			for i in x {
				let mut attribute_values = BTreeMap::new();
				for (attr_id, data) in i.attribute_values {
					match ItemAttrData::from_attr_data(&state, data).await {
						Ok(x) => {
							attribute_values.insert(attr_id, x);
						}

						Err(res) => return res,
					}
				}

				items.push(ItemlistItemInfo {
					id: i.id,
					class: i.class,
					attribute_values,
				})
			}

			return (
				StatusCode::OK,
				Json(ItemListResponse {
					total,
					skip: paginate.skip,
					items,
				}),
			)
				.into_response();
		}

		Err(ListItemsError::ClassNotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(ListItemsError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};
}
