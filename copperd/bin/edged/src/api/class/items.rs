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
	client::errors::{
		class::{ClassPrimaryAttributeError, GetClassError},
		dataset::GetDatasetError,
		item::{CountItemsError, GetItemError, ListItemsError},
	},
	AttrData, AttributeId, ClassId, ItemId,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Transaction};
use tracing::error;
use utoipa::{IntoParams, ToSchema};

//
// MARK: itemattrdata
//

/// Attribute data returned to the user
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type")]
pub(super) enum ItemAttrData {
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

		primary_attr: PrimaryAttrData,
	},
}

impl ItemAttrData {
	async fn from_attr_data<Client: DatabaseClient>(
		state: &RouterState<Client>,
		value: AttrData,
		trans: &mut Transaction<'_, sqlx::Postgres>,
	) -> Result<Self, Response> {
		Ok(match value {
			//
			// Easy
			//
			AttrData::Integer { value, .. } => Self::Integer { value },
			AttrData::Float { value, .. } => Self::Float { value },
			AttrData::Boolean { value } => Self::Boolean { value },

			AttrData::Text { value } => Self::Text {
				value: value.into(),
			},

			AttrData::Hash { data, .. } => Self::Hash {
				value: data.into_iter().map(|x| format!("{x:02X?}")).join(""),
			},

			//
			// MARK: blob
			//
			AttrData::Blob { bucket, key } => {
				let meta = match state
					.s3_client
					.get_object_metadata(bucket.as_str(), key.as_str())
					.await
				{
					Ok(x) => x,
					Err(error) => {
						error!(message = "Error in itemdb client", ?error);
						return Err((
							StatusCode::INTERNAL_SERVER_ERROR,
							Json("Internal server error"),
						)
							.into_response());
					}
				};

				ItemAttrData::Blob {
					mime: meta.mime.to_string(),
					size: meta.size,
				}
			}

			//
			// MARK: reference
			//
			AttrData::Reference { class, item } => Self::Reference {
				class,
				item,
				primary_attr: match state.itemdb_client.class_primary_attr(trans, class).await {
					Ok(primary_attr) => {
						if let Some(primary_attr) = primary_attr {
							let value = state.itemdb_client.get_item(trans, item).await;

							match value {
								Ok(value) => {
									PrimaryAttrData::from_attr_data(
										state,
										primary_attr.id,
										value
											.attribute_values
											.get(&primary_attr.id)
											.unwrap()
											.clone(),
									)
									.await?
								}

								Err(GetItemError::NotFound) => {
									return Err(StatusCode::NOT_FOUND.into_response())
								}

								Err(GetItemError::DbError(error)) => {
									error!(message = "Error in itemdb client", ?error);
									return Err((
										StatusCode::INTERNAL_SERVER_ERROR,
										Json("Internal server error"),
									)
										.into_response());
								}
							}
						} else {
							PrimaryAttrData::NotAvailable
						}
					}

					Err(ClassPrimaryAttributeError::NotFound) => {
						return Err(StatusCode::NOT_FOUND.into_response());
					}

					Err(ClassPrimaryAttributeError::DbError(error)) => {
						error!(message = "Error in itemdb client", ?error);
						return Err((
							StatusCode::INTERNAL_SERVER_ERROR,
							Json("Internal server error"),
						)
							.into_response());
					}
				},
			},
		})
	}
}

//
// MARK: primaryattrdata
//

// Almost identical to [`ItemAttrData`], but excluding references.
// Used inside the reference variant of [`ItemAttrData`].
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type")]
pub(super) enum PrimaryAttrData {
	NotAvailable,
	Text {
		#[schema(value_type = i64)]
		attr: AttributeId,
		value: String,
	},

	Integer {
		#[schema(value_type = i64)]
		attr: AttributeId,
		value: i64,
	},

	Float {
		#[schema(value_type = i64)]
		attr: AttributeId,
		value: f64,
	},

	Boolean {
		#[schema(value_type = i64)]
		attr: AttributeId,
		value: bool,
	},

	Hash {
		#[schema(value_type = i64)]
		attr: AttributeId,
		value: String,
	},

	Blob {
		#[schema(value_type = i64)]
		attr: AttributeId,

		mime: String,
		size: Option<i64>,
	},
}

impl PrimaryAttrData {
	async fn from_attr_data<Client: DatabaseClient>(
		state: &RouterState<Client>,
		attr: AttributeId,
		value: AttrData,
	) -> Result<Self, Response> {
		Ok(match value {
			//
			// Easy
			//
			AttrData::Integer { value, .. } => Self::Integer { value, attr },
			AttrData::Float { value, .. } => Self::Float { value, attr },
			AttrData::Boolean { value } => Self::Boolean { value, attr },

			AttrData::Text { value } => Self::Text {
				value: value.into(),
				attr,
			},

			AttrData::Hash { data, .. } => Self::Hash {
				value: data.into_iter().map(|x| format!("{x:02X?}")).join(""),
				attr,
			},

			//
			// MARK: blob
			//
			AttrData::Blob { bucket, key } => {
				let meta = match state
					.s3_client
					.get_object_metadata(bucket.as_str(), key.as_str())
					.await
				{
					Ok(x) => x,
					Err(error) => {
						error!(message = "Error in itemdb client", ?error);
						return Err((
							StatusCode::INTERNAL_SERVER_ERROR,
							Json("Internal server error"),
						)
							.into_response());
					}
				};

				Self::Blob {
					mime: meta.mime.to_string(),
					size: meta.size,
					attr,
				}
			}

			AttrData::Reference { .. } => {
				error!(message = "Tried to put a reference in a reference");
				return Err((
					StatusCode::INTERNAL_SERVER_ERROR,
					Json("Internal server error"),
				)
					.into_response());
			}
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

//
// MARK: route
//

/// List items in this class
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
pub(super) async fn list_items<Client: DatabaseClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client>>,
	Query(paginate): Query<PaginateParams>,
	Path(class_id): Path<i64>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let mut conn = match state.itemdb_client.new_connection().await {
		Ok(x) => x,
		Err(error) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	let mut trans = match conn.begin().await {
		Ok(y) => y,
		Err(error) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	let class = match state
		.itemdb_client
		.get_class(&mut trans, class_id.into())
		.await
	{
		Ok(x) => x,

		Err(GetClassError::NotFound) => return StatusCode::NOT_FOUND.into_response(),

		Err(GetClassError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	match state
		.itemdb_client
		.get_dataset(&mut trans, class.dataset)
		.await
	{
		Ok(x) => {
			// We can only modify our own class
			if x.owner != user.id {
				return (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response();
			}
		}

		// In theory unreachable, but possible with unlucky timing
		Err(GetDatasetError::NotFound) => {
			return (StatusCode::NOT_FOUND, Json("Dataset not found")).into_response()
		}

		Err(GetDatasetError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	let total = match state.itemdb_client.count_items(&mut trans, class.id).await {
		Ok(x) => x,

		// In theory unreachable, but possible with unlucky timing
		Err(CountItemsError::ClassNotFound) => {
			return (StatusCode::NOT_FOUND, Json("Class not found")).into_response()
		}

		Err(CountItemsError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};

	match state
		.itemdb_client
		.list_items(&mut trans, class.id, paginate.skip, paginate.count)
		.await
	{
		Ok(x) => {
			let mut items = Vec::new();

			for i in x {
				let mut attribute_values = BTreeMap::new();
				for (attr_id, data) in i.attribute_values {
					match ItemAttrData::from_attr_data(&state, data, &mut trans).await {
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

		Err(ListItemsError::ClassNotFound) => {
			return (StatusCode::NOT_FOUND, Json("Class not found")).into_response()
		}

		Err(ListItemsError::DbError(error)) => {
			error!(message = "Error in itemdb client", ?error);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json("Internal server error"),
			)
				.into_response();
		}
	};
}
