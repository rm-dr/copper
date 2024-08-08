use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use super::{class::ClassSelect, RouterState};

mod list;

use list::*;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(in crate::api) struct AttrSelect {
	#[serde(flatten)]
	pub class: ClassSelect,
	pub attr: String,
}

#[derive(OpenApi)]
#[openapi(
	tags(),
	paths(list_item),
	components(schemas(ItemListRequest, ItemListItem, ItemListData, ItemListResponse))
)]
pub(super) struct ItemApi;

pub(super) fn router() -> Router<RouterState> {
	Router::new().route("/list", get(list_item))
}
